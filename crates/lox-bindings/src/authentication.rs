use lox_core::authentication as core;
use std::sync::{Arc, Mutex};

/// Explicit CONFIG password digest, never the outer HMAC algorithm.
#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum WsxPasswordDigest {
    Sha1,
    Sha256,
}

impl From<WsxPasswordDigest> for core::WsxPasswordDigest {
    fn from(value: WsxPasswordDigest) -> Self {
        match value {
            WsxPasswordDigest::Sha1 => Self::Sha1,
            WsxPasswordDigest::Sha256 => Self::Sha256,
        }
    }
}

/// Core diagnostics contain no input, key, credential or token payload.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum AuthenticationHashError {
    #[error("{message}")]
    InvalidHexKey { message: String },
    #[error("{message}")]
    NonUtf8WsxKey { message: String },
    #[error("{message}")]
    Closed { message: String },
}

impl From<core::HashError> for AuthenticationHashError {
    fn from(value: core::HashError) -> Self {
        let message = value.to_string();
        match value {
            core::HashError::InvalidHexKey => Self::InvalidHexKey { message },
            core::HashError::NonUtf8WsxKey => Self::NonUtf8WsxKey { message },
        }
    }
}

/// Opaque secret: only to_hex reveals it. Closing makes it unavailable, but
/// neither this object nor immutable language strings promise memory erasure.
#[derive(Debug, uniffi::Object)]
pub struct AuthenticationHash {
    inner: Mutex<Option<core::AuthenticationHash>>,
}

impl AuthenticationHash {
    fn new(value: core::AuthenticationHash) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Some(value)),
        })
    }
}

#[uniffi::export]
impl AuthenticationHash {
    pub fn to_hex(&self) -> Result<String, AuthenticationHashError> {
        self.inner
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .map(core::AuthenticationHash::to_hex)
            .ok_or_else(|| AuthenticationHashError::Closed {
                message: "authentication hash is closed".into(),
            })
    }

    /// Idempotent logical disposal, independent of generated handle disposal.
    pub fn dispose(&self) {
        self.inner
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
    }
}

#[uniffi::export]
pub fn hash_app_credentials(
    user: String,
    password: String,
    key: Vec<u8>,
) -> Arc<AuthenticationHash> {
    AuthenticationHash::new(core::hash_credentials(
        core::CredentialProfile::AppRawPassword,
        &user,
        &password,
        &core::HashKey::from_bytes(&key),
    ))
}

#[uniffi::export]
pub fn hash_app_credentials_hex(
    user: String,
    password: String,
    key_hex: String,
) -> Result<Arc<AuthenticationHash>, AuthenticationHashError> {
    let key = core::HashKey::from_hex(&key_hex)?;
    Ok(AuthenticationHash::new(core::hash_credentials(
        core::CredentialProfile::AppRawPassword,
        &user,
        &password,
        &key,
    )))
}

#[uniffi::export]
pub fn hash_wsx_credentials(
    user: String,
    password: String,
    salt: String,
    password_digest: WsxPasswordDigest,
    key: Vec<u8>,
) -> Arc<AuthenticationHash> {
    AuthenticationHash::new(core::hash_credentials(
        core::CredentialProfile::ConfigWsx {
            salt: &salt,
            password_digest: password_digest.into(),
        },
        &user,
        &password,
        &core::HashKey::from_bytes(&key),
    ))
}

#[uniffi::export]
pub fn hash_wsx_credentials_hex(
    user: String,
    password: String,
    salt: String,
    password_digest: WsxPasswordDigest,
    key_hex: String,
) -> Result<Arc<AuthenticationHash>, AuthenticationHashError> {
    let key = core::HashKey::from_hex(&key_hex)?;
    Ok(AuthenticationHash::new(core::hash_credentials(
        core::CredentialProfile::ConfigWsx {
            salt: &salt,
            password_digest: password_digest.into(),
        },
        &user,
        &password,
        &key,
    )))
}

#[uniffi::export]
pub fn hash_app_token(token: String, key: Vec<u8>) -> Arc<AuthenticationHash> {
    AuthenticationHash::new(core::hash_app_token(
        &token,
        &core::HashKey::from_bytes(&key),
    ))
}

#[uniffi::export]
pub fn hash_app_token_hex(
    token: String,
    key_hex: String,
) -> Result<Arc<AuthenticationHash>, AuthenticationHashError> {
    let key = core::HashKey::from_hex(&key_hex)?;
    Ok(AuthenticationHash::new(core::hash_app_token(&token, &key)))
}

#[uniffi::export]
pub fn hash_wsx_autht(
    token: String,
    key: Vec<u8>,
) -> Result<Arc<AuthenticationHash>, AuthenticationHashError> {
    Ok(AuthenticationHash::new(core::hash_wsx_autht(
        &token,
        &core::HashKey::from_bytes(&key),
    )?))
}

#[uniffi::export]
pub fn hash_wsx_autht_hex(
    token: String,
    key_hex: String,
) -> Result<Arc<AuthenticationHash>, AuthenticationHashError> {
    let key = core::HashKey::from_hex(&key_hex)?;
    Ok(AuthenticationHash::new(core::hash_wsx_autht(&token, &key)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_profiles_preserve_core_known_answers_and_key_decoding() {
        let expected = [
            "88bfb35cc1a3ce183b6fcd66b3b5fe3acbdaba620c3ce041d866ce6d055e2bb2",
            "9d063ec0b204dd0a2368b4be99e052704579ada03e5b755e2fa4452646473790",
            "dffd58fcc0395555f0ea22c20b7f613cf1f2cb9103864f0f340c99bb3776a79c",
        ];
        let app =
            hash_app_credentials_hex("us:ér".into(), "päss:word".into(), "001122aaccee".into())
                .unwrap();
        assert_eq!(app.to_hex().unwrap(), expected[0]);
        for (digest, expected) in [
            (WsxPasswordDigest::Sha1, expected[1]),
            (WsxPasswordDigest::Sha256, expected[2]),
        ] {
            assert_eq!(
                hash_wsx_credentials_hex(
                    "us:ér".into(),
                    "päss:word".into(),
                    "s:älт".into(),
                    digest,
                    "001122aaccee".into(),
                )
                .unwrap()
                .to_hex()
                .unwrap(),
                expected
            );
        }
        assert_ne!(
            hash_app_token("tok".into(), b"ff".to_vec())
                .to_hex()
                .unwrap(),
            hash_app_token_hex("tok".into(), "ff".into())
                .unwrap()
                .to_hex()
                .unwrap()
        );
    }

    #[test]
    fn token_profiles_retain_utf8_boundary_casing_and_payload_free_errors() {
        let key = "clé:key".as_bytes().to_vec();
        let app = hash_app_token("jwt:тøken".into(), key.clone());
        let wsx = hash_wsx_autht("jwt:тøken".into(), key).unwrap();
        assert_eq!(
            app.to_hex().unwrap(),
            "4b59c58e38a81ab3dbe92b052651a678a6f5a175175ba01f1f1853849b25da89"
        );
        assert_eq!(wsx.to_hex().unwrap(), app.to_hex().unwrap().to_uppercase());
        assert!(hash_wsx_autht_hex("synthetic-secret-token".into(), "ff".into()).is_err());
        for invalid in ["a", "🌸", "synthetic-secret-key"] {
            let error =
                hash_app_token_hex("synthetic-secret-token".into(), invalid.into()).unwrap_err();
            assert!(matches!(
                error,
                AuthenticationHashError::InvalidHexKey { .. }
            ));
            assert_eq!(error.to_string(), "invalid hex authentication key");
            assert!(!format!("{error:?}").contains("synthetic-secret"));
        }
        let error = hash_wsx_autht("synthetic-secret-token".into(), vec![255]).unwrap_err();
        assert!(matches!(
            error,
            AuthenticationHashError::NonUtf8WsxKey { .. }
        ));
        assert_eq!(
            error.to_string(),
            "CONFIG wsx authentication key is not UTF-8"
        );
    }

    #[test]
    fn result_debug_is_redacted_and_disposal_is_idempotent() {
        let result = hash_app_token("synthetic-secret-token".into(), b"synthetic-key".to_vec());
        let revealed = result.to_hex().unwrap();
        assert!(!format!("{result:?}").contains(&revealed));
        result.dispose();
        result.dispose();
        assert!(matches!(
            result.to_hex(),
            Err(AuthenticationHashError::Closed { .. })
        ));
    }
}
