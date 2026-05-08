//! Loxone WebSocket client — used for token auth key exchange

use anyhow::Result;
use rustls::{ClientConfig, crypto::ring};
use std::sync::Arc;
use tokio_tungstenite::Connector;
use tokio_tungstenite::connect_async_tls_with_config;
use tokio_tungstenite::tungstenite::http::Request;

use rand::RngCore as _;

use crate::config::Config;

// ── TLS (accept all) ─────────────────────────────────────────────────────────

pub fn make_tls_config_pub() -> Arc<ClientConfig> {
    make_tls_config(false)
}

fn make_tls_config(verify_ssl: bool) -> Arc<ClientConfig> {
    let _ = ring::default_provider().install_default();
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let mut cfg = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    if !verify_ssl {
        cfg.dangerous()
            .set_certificate_verifier(Arc::new(NoCertVerifier));
    }
    cfg.enable_early_data = true;
    Arc::new(cfg)
}

#[derive(Debug)]
struct NoCertVerifier;
impl rustls::client::danger::ServerCertVerifier for NoCertVerifier {
    fn verify_server_cert(
        &self,
        _: &rustls::pki_types::CertificateDer,
        _: &[rustls::pki_types::CertificateDer],
        _: &rustls::pki_types::ServerName,
        _: &[u8],
        _: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &rustls::pki_types::CertificateDer,
        _: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(
        &self,
        _: &[u8],
        _: &rustls::pki_types::CertificateDer,
        _: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

// ── WS Client ────────────────────────────────────────────────────────────────

pub struct LoxWsClient {
    cfg: Config,
}

impl LoxWsClient {
    pub fn new(cfg: Config) -> Self {
        Self { cfg }
    }

    fn ws_url(&self) -> String {
        self.cfg
            .host
            .replace("https://", "wss://")
            .replace("http://", "ws://")
            .trim_end_matches('/')
            .to_string()
            + "/ws/rfc6455"
    }

    pub async fn connect_raw(
        &self,
    ) -> Result<(
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>,
    )> {
        let url = self.ws_url();
        let tls_cfg = make_tls_config(self.cfg.verify_ssl.unwrap_or(false));
        let basic = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:{}", self.cfg.user, self.cfg.pass),
        );
        let req = Request::builder()
            .uri(&url)
            .header("Authorization", format!("Basic {}", basic))
            .header(
                "Host",
                url.split("://")
                    .nth(1)
                    .unwrap_or("")
                    .split('/')
                    .next()
                    .unwrap_or(""),
            )
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_ws_key())
            .body(())?;
        connect_async_tls_with_config(req, None, false, Some(Connector::Rustls(tls_cfg)))
            .await
            .map_err(|e| anyhow::anyhow!("WS connect: {}", e))
    }
}

fn generate_ws_key() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
}

// ── Fast reload via /wsx ────────────────────────────────────────────────────

const MAGIC_FEEDBEEF: [u8; 4] = 0xFEED_BEEFu32.to_le_bytes(); // [0xEF, 0xBE, 0xED, 0xFE]

/// Connect to /wsx, perform handshake, and send 0x3A → 0x05 to trigger
/// a fast SPS reload (~4 seconds).  Used after FTP-uploading sps_new.zip.
///
/// Uses OpenSSL's `SSL_write` (via `ssl_write()`) so that each call produces
/// exactly **one** TLS ApplicationData record.  The hixie-76 → binary mode
/// transition requires the `\x00dev/loxone/start\xff` text frame and the
/// RC6-encrypted handshake to arrive as separate TLS records; higher-level
/// Rust TLS APIs (rustls `StreamOwned`, native-tls `TlsStream`) buffer
/// plaintext before encrypting, coalescing both writes into one record and
/// causing the Miniserver to stay in text mode.
#[cfg(not(windows))]
pub fn trigger_fast_reload(cfg: &Config) -> Result<()> {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    // Parse host and port from cfg.host
    let (host, port) = {
        let stripped = cfg
            .host
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/');
        if let Some((h, p)) = stripped.split_once(':') {
            let p = p.split('/').next().unwrap_or("443");
            (h.to_string(), p.parse::<u16>().unwrap_or(443))
        } else {
            let h = stripped.split('/').next().unwrap_or(stripped);
            (h.to_string(), 443u16)
        }
    };

    let client = crate::client::LoxClient::new(cfg.clone())?;

    // ── Auth: getkey2 → hash pw → sign → getjwt → autht ────────────────
    let autht = compute_wsx_autht(&client, cfg)?;

    // ── Uptime (seconds → ms + 2500 ms offset) ─────────────────────────
    let uptime_ms = {
        let r = client.get_text("jdev/sps/io/20123f74-0222-3d2f-ffff234d69b98eb1/state");
        match r {
            Ok(s) => {
                let secs: f64 = serde_json::from_str::<serde_json::Value>(&s)
                    .ok()
                    .and_then(|v| {
                        v["LL"]["value"]
                            .as_str()
                            .map(|s| s.to_string())
                            .or_else(|| v["LL"]["value"].as_f64().map(|f| f.to_string()))
                    })
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0);
                ((secs * 1000.0) as u32).wrapping_add(2500)
            }
            Err(_) => {
                eprintln!("⚠ Could not read SecondsBoot");
                0
            }
        }
    };

    // ── Build 64-byte RC6-encrypted handshake ───────────────────────────
    let hs = crate::wsx::build_handshake_with_ts(&cfg.user, &cfg.pass, "DEU", uptime_ms);

    // ── TLS connect via openssl crate ───────────────────────────────────
    //    ssl_write → SSL_write → exactly one TLS record per call
    let mut ctx_builder = openssl::ssl::SslConnector::builder(openssl::ssl::SslMethod::tls())?;
    if !cfg.verify_ssl.unwrap_or(false) {
        ctx_builder.set_verify(openssl::ssl::SslVerifyMode::NONE);
    }
    let connector = ctx_builder.build();
    let tcp = TcpStream::connect(format!("{host}:{port}"))?;
    tcp.set_nodelay(true)?;
    tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
    let mut tls = connector
        .connect(&host, tcp)
        .map_err(|e| anyhow::anyhow!("TLS: {e}"))?;

    // ── Write 1: HTTP upgrade (one TLS record) ─────────────────────────
    let upgrade = format!(
        "GET /wsx?autht={autht}&user={user} HTTP/1.1\r\n\
         Host: {host}:{port}\r\n\
         Upgrade: WebSocket\r\n\
         Connection: Upgrade\r\n\
         \r\n",
        autht = autht,
        user = cfg.user,
        host = host,
        port = port,
    );
    tls.ssl_write(upgrade.as_bytes())
        .map_err(|e| anyhow::anyhow!("upgrade write: {e}"))?;
    tls.flush()?;
    std::thread::sleep(Duration::from_millis(500));

    // Read 101 Switching Protocols
    let mut buf = [0u8; 4096];
    let mut total = 0;
    loop {
        let n = tls.read(&mut buf[total..])?;
        if n == 0 {
            break; // EOF — peer closed connection
        }
        total += n;
        if buf[..total].windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if total >= buf.len() {
            anyhow::bail!("/wsx upgrade response too large");
        }
    }
    if !String::from_utf8_lossy(&buf[..total]).contains("101") {
        anyhow::bail!("/wsx upgrade failed");
    }

    // ── Write 2: hixie-76 text frame (SEPARATE TLS record) ─────────────
    //    CRITICAL: must be its own SSL_write so the Miniserver sees it as a
    //    standalone text frame and transitions to binary mode.
    tls.ssl_write(b"\x00dev/loxone/start\xff")
        .map_err(|e| anyhow::anyhow!("start write: {e}"))?;
    tls.flush()?;
    std::thread::sleep(Duration::from_millis(500));

    // ── Write 3: RC6 handshake (SEPARATE TLS record) ────────────────────
    tls.ssl_write(&hs)
        .map_err(|e| anyhow::anyhow!("handshake write: {e}"))?;
    tls.flush()?;
    std::thread::sleep(Duration::from_millis(2000));

    // ── Read capabilities (0x01 = binary mode ack) ─────────────────────
    let n = tls.read(&mut buf).unwrap_or(0);
    if n == 0 || buf[0] != 0x01 {
        anyhow::bail!(
            "binary mode failed: 0x{:02X} (expected 0x01)",
            if n > 0 { buf[0] } else { 0 }
        );
    }

    // Drain stale messages (100 ms timeout)
    tls.get_ref()
        .set_read_timeout(Some(Duration::from_millis(100)))?;
    while tls.read(&mut buf).unwrap_or(0) > 0 {}
    tls.get_ref()
        .set_read_timeout(Some(Duration::from_secs(10)))?;

    // ── 0x3A PreSave ────────────────────────────────────────────────────
    tls.ssl_write(&build_binary_cmd(0x3A))
        .map_err(|e| anyhow::anyhow!("presave write: {e}"))?;
    tls.flush()?;
    std::thread::sleep(Duration::from_millis(2000));

    // Read 0x3A ack (first-byte check, same as Python)
    let n = tls.read(&mut buf).unwrap_or(0);
    if n == 0 || buf[0] != 0x3A {
        anyhow::bail!("PreSave failed: 0x{:02X}", if n > 0 { buf[0] } else { 0 });
    }

    // ── 0x05 PostSave → fast SPS reload ─────────────────────────────────
    tls.ssl_write(&build_binary_cmd(0x05))
        .map_err(|e| anyhow::anyhow!("postsave write: {e}"))?;
    tls.flush()?;

    drop(tls);
    Ok(())
}

#[cfg(windows)]
pub fn trigger_fast_reload(cfg: &Config) -> Result<()> {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let (host, port) = {
        let stripped = cfg
            .host
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/');
        if let Some((h, p)) = stripped.split_once(':') {
            let p = p.split('/').next().unwrap_or("443");
            (h.to_string(), p.parse::<u16>().unwrap_or(443))
        } else {
            let h = stripped.split('/').next().unwrap_or(stripped);
            (h.to_string(), 443u16)
        }
    };

    let client = crate::client::LoxClient::new(cfg.clone())?;
    let autht = compute_wsx_autht(&client, cfg)?;

    let uptime_ms = {
        let r = client.get_text("jdev/sps/io/20123f74-0222-3d2f-ffff234d69b98eb1/state");
        match r {
            Ok(s) => {
                let secs: f64 = serde_json::from_str::<serde_json::Value>(&s)
                    .ok()
                    .and_then(|v| {
                        v["LL"]["value"]
                            .as_str()
                            .map(|s| s.to_string())
                            .or_else(|| v["LL"]["value"].as_f64().map(|f| f.to_string()))
                    })
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0);
                ((secs * 1000.0) as u32).wrapping_add(2500)
            }
            Err(_) => {
                eprintln!("⚠ Could not read SecondsBoot");
                0
            }
        }
    };

    let hs = crate::wsx::build_handshake_with_ts(&cfg.user, &cfg.pass, "DEU", uptime_ms);

    let connector = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(!cfg.verify_ssl.unwrap_or(false))
        .build()?;
    let tcp = TcpStream::connect(format!("{host}:{port}"))?;
    tcp.set_nodelay(true)?;
    tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
    let mut tls = connector
        .connect(&host, tcp)
        .map_err(|e| anyhow::anyhow!("TLS: {e}"))?;

    let upgrade = format!(
        "GET /wsx?autht={autht}&user={user} HTTP/1.1\r\n\
         Host: {host}:{port}\r\n\
         Upgrade: WebSocket\r\n\
         Connection: Upgrade\r\n\
         \r\n",
        autht = autht,
        user = cfg.user,
        host = host,
        port = port,
    );
    tls.write_all(upgrade.as_bytes())
        .map_err(|e| anyhow::anyhow!("upgrade write: {e}"))?;
    tls.flush()?;
    std::thread::sleep(Duration::from_millis(500));

    let mut buf = [0u8; 4096];
    let mut total = 0;
    loop {
        let n = tls.read(&mut buf[total..])?;
        if n == 0 {
            break;
        }
        total += n;
        if buf[..total].windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if total >= buf.len() {
            anyhow::bail!("/wsx upgrade response too large");
        }
    }
    if !String::from_utf8_lossy(&buf[..total]).contains("101") {
        anyhow::bail!("/wsx upgrade failed");
    }

    tls.write_all(b"\x00dev/loxone/start\xff")
        .map_err(|e| anyhow::anyhow!("start write: {e}"))?;
    tls.flush()?;
    std::thread::sleep(Duration::from_millis(500));

    tls.write_all(&hs)
        .map_err(|e| anyhow::anyhow!("handshake write: {e}"))?;
    tls.flush()?;
    std::thread::sleep(Duration::from_millis(2000));

    let n = tls.read(&mut buf).unwrap_or(0);
    if n == 0 || buf[0] != 0x01 {
        anyhow::bail!(
            "binary mode failed: 0x{:02X} (expected 0x01)",
            if n > 0 { buf[0] } else { 0 }
        );
    }

    tls.get_ref()
        .set_read_timeout(Some(Duration::from_millis(100)))?;
    while tls.read(&mut buf).unwrap_or(0) > 0 {}
    tls.get_ref()
        .set_read_timeout(Some(Duration::from_secs(10)))?;

    tls.write_all(&build_binary_cmd(0x3A))
        .map_err(|e| anyhow::anyhow!("presave write: {e}"))?;
    tls.flush()?;
    std::thread::sleep(Duration::from_millis(2000));

    let n = tls.read(&mut buf).unwrap_or(0);
    if n == 0 || buf[0] != 0x3A {
        anyhow::bail!("PreSave failed: 0x{:02X}", if n > 0 { buf[0] } else { 0 });
    }

    tls.write_all(&build_binary_cmd(0x05))
        .map_err(|e| anyhow::anyhow!("postsave write: {e}"))?;
    tls.flush()?;

    drop(tls);
    Ok(())
}

/// Build a 16-byte binary /wsx command.
///
/// Matches Python: `struct.pack("<BBHII", cmd, 0, 0, 0, 0) + b"\xef\xbe\xed\xfe"`
fn build_binary_cmd(cmd: u8) -> [u8; 16] {
    let mut buf = [0u8; 16];
    buf[0] = cmd;
    buf[12..16].copy_from_slice(&MAGIC_FEEDBEEF);
    buf
}

/// Compute the HMAC-SHA256 auth token for the /wsx endpoint.
///
/// Mirrors the Python: getkey2 → hash pw → HMAC sign → getjwt → autht.
fn compute_wsx_autht(client: &crate::client::LoxClient, cfg: &Config) -> Result<String> {
    // 1. getkey2 — one-time HMAC key + password salt
    let gk2_resp = client.get_text(&format!("jdev/sys/getkey2/{}", cfg.user))?;
    let gk2: serde_json::Value = serde_json::from_str(&gk2_resp)?;
    // The value may be a JSON string that needs a second parse
    let gk2_val = {
        let v = &gk2["LL"]["value"];
        if let Some(s) = v.as_str() {
            serde_json::from_str(s)?
        } else {
            v.clone()
        }
    };
    let key_hex = gk2_val["key"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("missing key in getkey2"))?;
    let salt = gk2_val["salt"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("missing salt in getkey2"))?;
    let hash_alg = gk2_val
        .get("hashAlg")
        .and_then(|v| v.as_str())
        .unwrap_or("SHA1");

    // 2. Hash password: HASH("password:salt") → uppercase hex
    let pw_hash = if hash_alg == "SHA256" {
        use sha2::{Digest, Sha256};
        hex::encode_upper(Sha256::digest(format!("{}:{}", cfg.pass, salt).as_bytes()))
    } else {
        use sha1::Digest as _;
        hex::encode_upper(sha1::Sha1::digest(
            format!("{}:{}", cfg.pass, salt).as_bytes(),
        ))
    };

    // 3. Sign: HMAC-SHA256(key, "user:pw_hash")
    let sig = {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<sha2::Sha256>;
        let mut mac = HmacSha256::new_from_slice(&hex::decode(key_hex)?)?;
        mac.update(format!("{}:{}", cfg.user, pw_hash).as_bytes());
        hex::encode(mac.finalize().into_bytes())
    };

    // 4. getjwt — permission 8 = CONFIG
    let jwt_resp = client.get_text(&format!(
        "jdev/sys/getjwt/{}/{}/8/{}/lox-cli",
        sig,
        cfg.user,
        uuid::Uuid::new_v4()
    ))?;
    let jwt_val: serde_json::Value = serde_json::from_str(&jwt_resp)?;
    let jwt_token = jwt_val["LL"]["value"]["token"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("no JWT token"))?;
    let jwt_key_hex = jwt_val["LL"]["value"]["key"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("no JWT key"))?;

    // 5. autht = HMAC-SHA256(hex_decode(jwt_key).as_ascii(), jwt_token)
    //    Python: hmac.new(bytes.fromhex(key).decode("ascii").encode(), token, sha256)
    let jwt_key_ascii = String::from_utf8(hex::decode(jwt_key_hex)?)?;
    let autht = {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<sha2::Sha256>;
        let mut mac = HmacSha256::new_from_slice(jwt_key_ascii.as_bytes())?;
        mac.update(jwt_token.as_bytes());
        hex::encode_upper(mac.finalize().into_bytes())
    };

    Ok(autht)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_ws_key_length() {
        let key = generate_ws_key();
        assert_eq!(key.len(), 24);
    }

    #[test]
    fn test_generate_ws_key_unique() {
        let k1 = generate_ws_key();
        let k2 = generate_ws_key();
        assert_ne!(k1, k2);
    }
}
