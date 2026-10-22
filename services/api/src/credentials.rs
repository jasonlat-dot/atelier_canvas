//! AES-256-GCM authenticated envelopes. Key material is injected by the host, never stored in DB.
use crate::{ApiError, ApiResult};
use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm,
};
use axum::http::StatusCode;
use base64::{engine::general_purpose::STANDARD, Engine};
use uuid::Uuid;

pub(crate) struct CredentialCipher(Aes256Gcm);
fn unavailable() -> ApiError {
    ApiError(
        StatusCode::SERVICE_UNAVAILABLE,
        "credential encryption unavailable; configure ATELIER_CREDENTIAL_KEY".into(),
    )
}
impl CredentialCipher {
    pub(crate) fn from_base64(key: &str) -> ApiResult<Self> {
        let bytes = STANDARD.decode(key.trim()).map_err(|_| unavailable())?;
        if bytes.len() != 32 {
            return Err(unavailable());
        }
        Ok(Self(
            Aes256Gcm::new_from_slice(&bytes).map_err(|_| unavailable())?,
        ))
    }
    pub(crate) fn from_env() -> ApiResult<Option<std::sync::Arc<Self>>> {
        match std::env::var("ATELIER_CREDENTIAL_KEY") {
            Ok(value) => Ok(Some(std::sync::Arc::new(Self::from_base64(&value)?))),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(_) => Err(unavailable()),
        }
    }
    pub(crate) fn encrypt(&self, id: Uuid, text: &str) -> ApiResult<Vec<u8>> {
        let mut nonce = [0u8; 12];
        getrandom::fill(&mut nonce).map_err(|_| unavailable())?;
        let bytes = self
            .0
            .encrypt(
                &nonce.into(),
                Payload {
                    msg: text.as_bytes(),
                    aad: id.as_bytes(),
                },
            )
            .map_err(|_| unavailable())?;
        let mut result = vec![1];
        result.extend(nonce);
        result.extend(bytes);
        Ok(result)
    }
    pub(crate) fn decrypt(&self, id: Uuid, bytes: &[u8]) -> ApiResult<String> {
        if bytes.len() < 29 || bytes[0] != 1 {
            return Err(unavailable());
        }
        let nonce: [u8; 12] = bytes[1..13].try_into().map_err(|_| unavailable())?;
        let plain = self
            .0
            .decrypt(
                &nonce.into(),
                Payload {
                    msg: &bytes[13..],
                    aad: id.as_bytes(),
                },
            )
            .map_err(|_| unavailable())?;
        String::from_utf8(plain).map_err(|_| unavailable())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authenticated_envelope_is_random_bound_and_persistent() {
        let key = STANDARD.encode([7u8; 32]);
        let cipher = CredentialCipher::from_base64(&key).unwrap();
        let id = Uuid::new_v4();
        let a = cipher.encrypt(id, "synthetic-test-credential").unwrap();
        let b = cipher.encrypt(id, "synthetic-test-credential").unwrap();
        assert_ne!(a, b);
        assert!(!String::from_utf8_lossy(&a).contains("synthetic-test-credential"));
        let reopened = CredentialCipher::from_base64(&key).unwrap();
        assert_eq!(
            reopened.decrypt(id, &a).unwrap(),
            "synthetic-test-credential"
        );
        assert!(reopened.decrypt(Uuid::new_v4(), &a).is_err());
        let mut altered = a;
        altered[13] ^= 1;
        assert!(reopened.decrypt(id, &altered).is_err());
    }
    #[test]
    fn invalid_key_and_envelope_fail_without_plaintext() {
        assert!(CredentialCipher::from_base64("invalid").is_err());
        assert!(CredentialCipher::from_base64(&STANDARD.encode([0u8; 31])).is_err());
        let cipher = CredentialCipher::from_base64(&STANDARD.encode([8u8; 32])).unwrap();
        assert!(cipher.decrypt(Uuid::new_v4(), b"plaintext").is_err());
    }
}
