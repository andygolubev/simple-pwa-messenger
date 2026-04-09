use wasm_bindgen::JsValue;

#[derive(Debug)]
pub enum CryptoError {
    InvalidInput(String),
    DecodeError(String),
    EncryptionError,
    DecryptionError,
    InvalidSignature,
    SessionError(String),
}

impl CryptoError {
    pub fn to_js_value(&self) -> JsValue {
        match self {
            CryptoError::InvalidInput(msg)
            | CryptoError::DecodeError(msg)
            | CryptoError::SessionError(msg) => JsValue::from_str(msg),
            CryptoError::EncryptionError => JsValue::from_str("encryption failed"),
            CryptoError::DecryptionError => JsValue::from_str("decryption failed"),
            CryptoError::InvalidSignature => JsValue::from_str("signature verification failed"),
        }
    }
}
