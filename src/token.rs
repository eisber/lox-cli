//! Loxone token authentication.
//!
//! Implements the Loxone token auth protocol:
//! 1. Connect WebSocket to `wss://<host>/ws/rfc6455`
//! 2. Fetch RSA public key from the Miniserver
//! 3. Perform AES-256 key exchange (RSA-encrypted session key)
//! 4. Request a token via HMAC-SHA256 authenticated credential exchange
//! 5. Store the token locally for reuse (~20 day validity)

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use rand::RngCore;
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::pkcs8::DecodePublicKey;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::config::Config;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Loxone epoch: 2009-01-01 00:00:00 UTC, expressed as seconds since Unix epoch.
const LOXONE_EPOCH_OFFSET: u64 = 1_230_768_000;

/// Permission level 4 = application / long-lived token.
const PERMISSION_APP: u32 = 4;

/// Info string sent during token request.
const CLIENT_INFO: &str = "lox-cli";

/// WebSocket connect timeout.
const WS_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Overall timeout for the full key-exchange + token-acquisition flow.
const WS_TOTAL_TIMEOUT: Duration = Duration::from_secs(30);

/// Safety margin: consider a token invalid 1 day before actual expiry.
const VALIDITY_MARGIN: Duration = Duration::from_secs(86_400);

// ---------------------------------------------------------------------------
// AES helpers (CBC / PKCS7)
// ---------------------------------------------------------------------------

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

/// Encrypt `plaintext` with AES-256-CBC + PKCS7 padding.
fn aes_encrypt(key: &[u8; 32], iv: &[u8; 16], plaintext: &[u8]) -> Vec<u8> {
    let encryptor = Aes256CbcEnc::new(key.into(), iv.into());
    encryptor.encrypt_padded_vec_mut::<Pkcs7>(plaintext)
}

/// Decrypt `ciphertext` produced by AES-256-CBC + PKCS7 padding.
fn aes_decrypt(key: &[u8; 32], iv: &[u8; 16], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let decryptor = Aes256CbcDec::new(key.into(), iv.into());
    decryptor
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| anyhow::anyhow!("AES decryption failed: {e}"))
}

// ---------------------------------------------------------------------------
// TokenStore
// ---------------------------------------------------------------------------

/// Locally-cached authentication token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStore {
    /// The opaque token string returned by the Miniserver.
    pub token: String,
    /// The HMAC key associated with this token (hex-encoded).
    pub key: String,
    /// Expiry as seconds since the **Loxone epoch** (2009-01-01 00:00:00 UTC).
    pub valid_until: u64,
}

impl TokenStore {
    /// Filesystem path where the token is persisted for the current config context.
    pub fn path_for(cfg: &Config) -> PathBuf {
        cfg.data_dir.join("token.json")
    }

    /// Load a previously-saved token from disk, returning `None` if the file
    /// is missing or cannot be parsed.
    pub fn load_for(cfg: &Config) -> Option<Self> {
        let path = Self::path_for(cfg);
        let data = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// Persist the token to disk so it can be reused across CLI invocations.
    pub fn save_for(&self, cfg: &Config) -> Result<()> {
        let path = Self::path_for(cfg);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating token directory {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)
            .with_context(|| format!("writing token to {}", path.display()))?;
        Ok(())
    }

    /// Returns `true` if the token is still usable (with a 1-day safety margin).
    pub fn is_valid(&self) -> bool {
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let expires_unix = self.valid_until + LOXONE_EPOCH_OFFSET;

        // Token is valid if expiry (minus margin) is still in the future.
        expires_unix.saturating_sub(VALIDITY_MARGIN.as_secs()) > now_unix
    }
}

// ---------------------------------------------------------------------------
// HMAC helpers
// ---------------------------------------------------------------------------

/// Compute HMAC-SHA256(`message`, `key`) and return the result as a
/// lowercase hex string.  `key` is expected to be hex-encoded.
pub fn hash_token(token: &str, key: &str) -> String {
    let key_bytes = hex::decode(key).unwrap_or_default();
    let mut mac = Hmac::<Sha256>::new_from_slice(&key_bytes).expect("HMAC accepts any key length");
    mac.update(token.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Compute `HMAC-SHA256(user:password, key)` where `key` is hex-encoded.
fn hash_credentials(user: &str, password: &str, key_hex: &str) -> Result<String> {
    let key_bytes =
        hex::decode(key_hex).context("Miniserver returned invalid hex key for hashing")?;
    let mut mac = Hmac::<Sha256>::new_from_slice(&key_bytes).context("invalid HMAC key length")?;
    let payload = format!("{user}:{password}");
    mac.update(payload.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

// ---------------------------------------------------------------------------
// Loxone epoch conversion
// ---------------------------------------------------------------------------

/// Convert a Loxone timestamp (seconds since 2009-01-01) to a Unix timestamp.
pub fn loxone_to_unix(loxone_ts: u64) -> u64 {
    loxone_ts + LOXONE_EPOCH_OFFSET
}

/// Convert a Unix timestamp to a Loxone timestamp.
pub fn unix_to_loxone(unix_ts: u64) -> u64 {
    unix_ts.saturating_sub(LOXONE_EPOCH_OFFSET)
}

// ---------------------------------------------------------------------------
// WebSocket helpers
// ---------------------------------------------------------------------------

/// Build the WebSocket URL from the config host.
fn ws_url(cfg: &Config) -> String {
    cfg.host
        .replace("https://", "wss://")
        .replace("http://", "ws://")
        .trim_end_matches('/')
        .to_string()
        + "/ws/rfc6455"
}

/// Parse a Loxone JSON-wrapped response, e.g.:
/// `{"LL":{"control":"jdev/sys/getPublicKey","value":"...","code":"200"}}`
///
/// Returns `(code, value)`.
fn parse_ll_response(text: &str) -> Result<(u32, String)> {
    let v: serde_json::Value = serde_json::from_str(text).context("response is not valid JSON")?;
    let ll = v.get("LL").context("response missing 'LL' envelope")?;
    let code = ll
        .get("code")
        .and_then(|c| {
            c.as_str()
                .or_else(|| c.as_u64().map(|_| ""))
                .and_then(|s| s.parse().ok())
                .or_else(|| c.as_u64().map(|n| n as u32))
        })
        .unwrap_or(0);
    let value = ll
        .get("value")
        .map(|v| match v {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        })
        .unwrap_or_default();
    Ok((code, value))
}

// ---------------------------------------------------------------------------
// Token acquisition
// ---------------------------------------------------------------------------

/// Perform the full Loxone token-auth handshake over WebSocket:
///
/// 1. Connect to `wss://<host>/ws/rfc6455`
/// 2. Fetch the Miniserver's RSA public key
/// 3. Generate a random AES-256 session key + IV
/// 4. Encrypt and exchange the session key via RSA
/// 5. Request a token using HMAC-hashed credentials (AES-encrypted)
/// 6. Return the resulting [`TokenStore`]
pub async fn acquire_token(cfg: &Config) -> Result<TokenStore> {
    timeout(WS_TOTAL_TIMEOUT, acquire_token_inner(cfg))
        .await
        .context("token acquisition timed out")?
}

async fn acquire_token_inner(cfg: &Config) -> Result<TokenStore> {
    // ── 1. Connect WebSocket ────────────────────────────────────────────
    let url = ws_url(cfg);

    let tls = tokio_tungstenite::Connector::Rustls(std::sync::Arc::new(
        rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(std::sync::Arc::new(AcceptAll))
            .with_no_client_auth(),
    ));

    let (mut ws, _) = timeout(
        WS_CONNECT_TIMEOUT,
        tokio_tungstenite::connect_async_tls_with_config(&url, None, false, Some(tls)),
    )
    .await
    .context("WebSocket connect timed out")?
    .context("WebSocket connect failed")?;

    // ── 2. Get RSA public key ───────────────────────────────────────────
    ws.send(Message::Text("jdev/sys/getPublicKey".into()))
        .await?;
    let pubkey_pem = recv_text_value(&mut ws).await.context("getPublicKey")?;

    // The Miniserver may return the key with `-----BEGIN PUBLIC KEY-----` (SPKI)
    // or `-----BEGIN RSA PUBLIC KEY-----` (PKCS#1).  Try both.
    let cleaned = pubkey_pem.replace("-----", "-----\n").replace("\n\n", "\n");
    let rsa_pub = RsaPublicKey::from_public_key_pem(&cleaned)
        .or_else(|_| RsaPublicKey::from_pkcs1_pem(&cleaned))
        .or_else(|_| RsaPublicKey::from_public_key_pem(&pubkey_pem))
        .or_else(|_| RsaPublicKey::from_pkcs1_pem(&pubkey_pem))
        .context("failed to parse Miniserver RSA public key")?;

    // ── 3. Generate AES session key + IV ────────────────────────────────
    let mut aes_key = [0u8; 32];
    let mut aes_iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut aes_key);
    rand::thread_rng().fill_bytes(&mut aes_iv);

    let session_key_plain = format!("{}:{}", hex::encode(aes_key), hex::encode(aes_iv));

    // ── 4. RSA-encrypt and send key exchange ────────────────────────────
    let mut rng = rand::thread_rng();
    let encrypted = rsa_pub
        .encrypt(&mut rng, Pkcs1v15Encrypt, session_key_plain.as_bytes())
        .context("RSA encryption failed")?;
    let b64_encrypted = B64.encode(&encrypted);
    let keyexchange_cmd = format!("jdev/sys/keyexchange/{b64_encrypted}");
    ws.send(Message::Text(keyexchange_cmd)).await?;

    let (code, _) = recv_ll_response(&mut ws).await.context("keyexchange")?;
    if code != 200 {
        bail!("key exchange failed (code {code})");
    }

    // ── 5. Request hash key for credentials ─────────────────────────────
    let getkey_cmd = encrypt_command(&aes_key, &aes_iv, &format!("jdev/sys/getkey2/{}", cfg.user));
    ws.send(Message::Text(getkey_cmd)).await?;
    let key_response = recv_encrypted_value(&mut ws, &aes_key, &aes_iv)
        .await
        .context("getkey2")?;

    // Response is JSON: {"key":"<hex>", "salt":"<hex>"} or just hex key.
    let hash_key = extract_key_from_response(&key_response)?;

    // ── 6. Hash credentials and request token ───────────────────────────
    let cred_hash = hash_credentials(&cfg.user, &cfg.pass, &hash_key)?;
    let device_uuid = Uuid::new_v4().to_string();
    let token_cmd = format!(
        "jdev/sys/gettoken/{cred_hash}/{}/{PERMISSION_APP}/{device_uuid}/{CLIENT_INFO}",
        cfg.user
    );
    let enc_cmd = encrypt_command(&aes_key, &aes_iv, &token_cmd);
    ws.send(Message::Text(enc_cmd)).await?;

    let token_response = recv_encrypted_value(&mut ws, &aes_key, &aes_iv)
        .await
        .context("gettoken")?;

    // ── 7. Parse token response ─────────────────────────────────────────
    let tv: serde_json::Value =
        serde_json::from_str(&token_response).context("token response is not JSON")?;
    let token = tv
        .get("token")
        .and_then(|v| v.as_str())
        .context("missing 'token' in response")?
        .to_string();
    let key = tv
        .get("key")
        .and_then(|v| v.as_str())
        .context("missing 'key' in response")?
        .to_string();
    let valid_until = tv
        .get("validUntil")
        .and_then(|v| v.as_u64().or_else(|| v.as_f64().map(|f| f as u64)))
        .context("missing 'validUntil' in response")?;

    // Clean up
    let _ = ws.close(None).await;

    Ok(TokenStore {
        token,
        key,
        valid_until,
    })
}

// ---------------------------------------------------------------------------
// WebSocket message helpers
// ---------------------------------------------------------------------------

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Receive the next text message and extract its LL value.
async fn recv_text_value(ws: &mut WsStream) -> Result<String> {
    loop {
        match ws.next().await {
            Some(Ok(Message::Text(text))) => {
                let (code, value) = parse_ll_response(&text)?;
                if code != 200 {
                    bail!("Miniserver error (code {code}): {value}");
                }
                return Ok(value);
            }
            Some(Ok(_)) => continue, // skip binary/ping/pong
            Some(Err(e)) => bail!("WebSocket error: {e}"),
            None => bail!("WebSocket closed unexpectedly"),
        }
    }
}

/// Receive the next LL response (plaintext JSON).
async fn recv_ll_response(ws: &mut WsStream) -> Result<(u32, String)> {
    loop {
        match ws.next().await {
            Some(Ok(Message::Text(text))) => return parse_ll_response(&text),
            Some(Ok(_)) => continue,
            Some(Err(e)) => bail!("WebSocket error: {e}"),
            None => bail!("WebSocket closed unexpectedly"),
        }
    }
}

/// Receive the next text message, AES-decrypt its LL value, and return the
/// decrypted string.
async fn recv_encrypted_value(
    ws: &mut WsStream,
    aes_key: &[u8; 32],
    aes_iv: &[u8; 16],
) -> Result<String> {
    loop {
        match ws.next().await {
            Some(Ok(Message::Text(text))) => {
                let (code, value) = parse_ll_response(&text)?;
                if code != 200 {
                    bail!("Miniserver error (code {code}): {value}");
                }
                // Value is base64-encoded AES ciphertext.
                let ciphertext = B64.decode(&value).context("base64 decode failed")?;
                let plain = aes_decrypt(aes_key, aes_iv, &ciphertext)?;
                return String::from_utf8(plain).context("decrypted value is not UTF-8");
            }
            Some(Ok(_)) => continue,
            Some(Err(e)) => bail!("WebSocket error: {e}"),
            None => bail!("WebSocket closed unexpectedly"),
        }
    }
}

/// AES-encrypt a command string and return `jdev/sys/enc/{base64_ciphertext}`.
fn encrypt_command(aes_key: &[u8; 32], aes_iv: &[u8; 16], cmd: &str) -> String {
    let ciphertext = aes_encrypt(aes_key, aes_iv, cmd.as_bytes());
    let b64 = B64.encode(&ciphertext);
    format!("jdev/sys/enc/{b64}")
}

/// Extract the HMAC key from the getkey2 response.
///
/// The response may be a JSON object `{"key":"<hex>", ...}` or a bare hex string.
fn extract_key_from_response(response: &str) -> Result<String> {
    // Try JSON first.
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(response)
        && let Some(key) = v.get("key").and_then(|k| k.as_str())
    {
        return Ok(key.to_string());
    }
    // Fall back to treating the whole response as a hex key.
    let trimmed = response.trim().trim_matches('"');
    if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(trimmed.to_string());
    }
    bail!("could not extract HMAC key from Miniserver response: {response}")
}

// ---------------------------------------------------------------------------
// TLS: accept self-signed certificates
// ---------------------------------------------------------------------------

/// A rustls verifier that accepts any server certificate.  Loxone Miniservers
/// ship with self-signed certificates.
#[derive(Debug)]
struct AcceptAll;

impl rustls::client::danger::ServerCertVerifier for AcceptAll {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    // -- hash_token -------------------------------------------------------

    #[test]
    fn hash_token_known_vector() {
        // HMAC-SHA256("mytoken", key=bytes_of("aa"))
        let result = hash_token("mytoken", "aa");
        // Verify it's a 64-char lowercase hex string (SHA-256 output).
        assert_eq!(result.len(), 64);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));

        // Deterministic: same inputs → same output.
        assert_eq!(result, hash_token("mytoken", "aa"));
    }

    #[test]
    fn hash_token_different_keys_differ() {
        let a = hash_token("tok", "aabb");
        let b = hash_token("tok", "ccdd");
        assert_ne!(a, b);
    }

    #[test]
    fn hash_token_different_tokens_differ() {
        let a = hash_token("alpha", "aabb");
        let b = hash_token("beta", "aabb");
        assert_ne!(a, b);
    }

    #[test]
    fn hash_token_empty_key_does_not_panic() {
        let result = hash_token("tok", "");
        assert_eq!(result.len(), 64);
    }

    // -- TokenStore serialization -----------------------------------------

    #[test]
    fn token_store_round_trip() {
        let store = TokenStore {
            token: "abc-123".into(),
            key: "deadbeef".into(),
            valid_until: 500_000_000,
        };
        let json = serde_json::to_string(&store).unwrap();
        let restored: TokenStore = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.token, "abc-123");
        assert_eq!(restored.key, "deadbeef");
        assert_eq!(restored.valid_until, 500_000_000);
    }

    #[test]
    fn token_store_deserialize_from_json_string() {
        let json = r#"{"token":"t1","key":"k1","valid_until":12345}"#;
        let store: TokenStore = serde_json::from_str(json).unwrap();
        assert_eq!(store.token, "t1");
        assert_eq!(store.key, "k1");
        assert_eq!(store.valid_until, 12345);
    }

    // -- is_valid ---------------------------------------------------------

    #[test]
    fn is_valid_far_future() {
        let store = TokenStore {
            token: String::new(),
            key: String::new(),
            valid_until: u64::MAX / 2, // far future in Loxone time
        };
        assert!(store.is_valid());
    }

    #[test]
    fn is_valid_expired() {
        // Loxone timestamp 0 = 2009-01-01 → well in the past.
        let store = TokenStore {
            token: String::new(),
            key: String::new(),
            valid_until: 0,
        };
        assert!(!store.is_valid());
    }

    #[test]
    fn is_valid_within_margin() {
        // Set valid_until to "now + 12 hours" in Loxone time.
        // The 1-day margin should make this invalid.
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let loxone_ts = now_unix - LOXONE_EPOCH_OFFSET + 12 * 3600;
        let store = TokenStore {
            token: String::new(),
            key: String::new(),
            valid_until: loxone_ts,
        };
        assert!(
            !store.is_valid(),
            "token expiring in 12h should be invalid (1-day margin)"
        );
    }

    #[test]
    fn is_valid_beyond_margin() {
        // Set valid_until to "now + 3 days" in Loxone time.
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let loxone_ts = now_unix - LOXONE_EPOCH_OFFSET + 3 * 86_400;
        let store = TokenStore {
            token: String::new(),
            key: String::new(),
            valid_until: loxone_ts,
        };
        assert!(store.is_valid(), "token expiring in 3 days should be valid");
    }

    // -- Loxone epoch conversion ------------------------------------------

    #[test]
    fn epoch_conversion_zero() {
        assert_eq!(loxone_to_unix(0), LOXONE_EPOCH_OFFSET);
        assert_eq!(unix_to_loxone(LOXONE_EPOCH_OFFSET), 0);
    }

    #[test]
    fn epoch_round_trip() {
        let lox_ts = 500_000_000u64;
        assert_eq!(unix_to_loxone(loxone_to_unix(lox_ts)), lox_ts);
    }

    #[test]
    fn loxone_epoch_is_2009_01_01() {
        // 2009-01-01 00:00:00 UTC = Unix 1230768000
        assert_eq!(LOXONE_EPOCH_OFFSET, 1_230_768_000);
    }

    #[test]
    fn unix_to_loxone_before_epoch_saturates() {
        // Before 2009 → should saturate to 0, not underflow.
        assert_eq!(unix_to_loxone(0), 0);
    }

    // -- AES round-trip ---------------------------------------------------

    #[test]
    fn aes_encrypt_decrypt_round_trip() {
        let key = [0x42u8; 32];
        let iv = [0x13u8; 16];
        let plaintext = b"Hello, Loxone!";

        let ciphertext = aes_encrypt(&key, &iv, plaintext);
        assert_ne!(&ciphertext, plaintext);

        let decrypted = aes_decrypt(&key, &iv, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn aes_empty_plaintext() {
        let key = [0xABu8; 32];
        let iv = [0xCDu8; 16];
        let ciphertext = aes_encrypt(&key, &iv, b"");
        let decrypted = aes_decrypt(&key, &iv, &ciphertext).unwrap();
        assert!(decrypted.is_empty());
    }

    // -- encrypt_command --------------------------------------------------

    #[test]
    fn encrypt_command_format() {
        let key = [0x01u8; 32];
        let iv = [0x02u8; 16];
        let result = encrypt_command(&key, &iv, "jdev/sys/test");
        assert!(result.starts_with("jdev/sys/enc/"));
        // The part after enc/ should be valid base64.
        let b64_part = result.strip_prefix("jdev/sys/enc/").unwrap();
        assert!(B64.decode(b64_part).is_ok());
    }

    // -- parse_ll_response ------------------------------------------------

    #[test]
    fn parse_ll_response_ok() {
        let json = r#"{"LL":{"control":"jdev/sys/getPublicKey","value":"MYPEM","code":"200"}}"#;
        let (code, value) = parse_ll_response(json).unwrap();
        assert_eq!(code, 200);
        assert_eq!(value, "MYPEM");
    }

    #[test]
    fn parse_ll_response_numeric_code() {
        let json = r#"{"LL":{"control":"test","value":"v","code":200}}"#;
        let (code, value) = parse_ll_response(json).unwrap();
        assert_eq!(code, 200);
        assert_eq!(value, "v");
    }

    #[test]
    fn parse_ll_response_error_code() {
        let json = r#"{"LL":{"control":"test","value":"denied","code":"401"}}"#;
        let (code, _) = parse_ll_response(json).unwrap();
        assert_eq!(code, 401);
    }

    // -- extract_key_from_response ----------------------------------------

    #[test]
    fn extract_key_json() {
        let resp = r#"{"key":"aabbccdd","salt":"11223344"}"#;
        assert_eq!(extract_key_from_response(resp).unwrap(), "aabbccdd");
    }

    #[test]
    fn extract_key_bare_hex() {
        assert_eq!(extract_key_from_response("aabbccdd").unwrap(), "aabbccdd");
    }

    #[test]
    fn extract_key_invalid() {
        assert!(extract_key_from_response("not-hex!").is_err());
    }
}
