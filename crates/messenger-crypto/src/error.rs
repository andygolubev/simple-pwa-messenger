use std::fmt;
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub enum CryptoError {
    InvalidBase64(&'static str),
    InvalidLength(&'static str),
    InvalidKeyMaterial(&'static str),
    SignatureVerificationFailed,
    EncryptionFailed,
    DecryptionFailed,
    Serialization(String),
    InvalidInput(String),
    InvalidEncoding(String),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::InvalidBase64(field) => write!(f, "invalid base64 in field: {field}"),
            CryptoError::InvalidLength(field) => write!(f, "invalid length in field: {field}"),
            CryptoError::InvalidKeyMaterial(field) => write!(f, "invalid key material: {field}"),
            CryptoError::SignatureVerificationFailed => write!(f, "signature verification failed"),
            CryptoError::EncryptionFailed => write!(f, "encryption failed"),
            CryptoError::DecryptionFailed => write!(f, "decryption failed"),
            CryptoError::Serialization(msg) => write!(f, "serialization error: {msg}"),
            CryptoError::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            CryptoError::InvalidEncoding(msg) => write!(f, "invalid encoding: {msg}"),
        }
    }
}

impl From<CryptoError> for JsValue {
    fn from(err: CryptoError) -> Self {
        JsValue::from_str(&err.to_string())
    }
}
