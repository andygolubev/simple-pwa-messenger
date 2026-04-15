use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::{Deserialize, Serialize};

use crate::error::CryptoError;

/// In-memory ratchet session state.
/// For sequential (non-DH) ratchet use; key fields are raw bytes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RatchetSessionState {
    pub root_key: [u8; 32],
    pub send_chain_key: [u8; 32],
    pub recv_chain_key: [u8; 32],
    pub message_number: u32,
    pub previous_chain_length: u32,
    /// Ratchet DH public key (serialized; may be empty for symmetric-only sessions).
    pub ratchet_public_key: Vec<u8>,
}

impl RatchetSessionState {
    /// Create a new session state from an X3DH shared secret.
    /// The initiator and responder swap send/recv chain keys.
    pub fn from_shared_secret(shared_secret: &[u8; 32], is_initiator: bool) -> Self {
        let root = hkdf_expand(shared_secret, b"messenger:root");
        let c1 = hkdf_expand(&root, b"messenger:chain:1");
        let c2 = hkdf_expand(&root, b"messenger:chain:2");

        let (send_chain_key, recv_chain_key) = if is_initiator {
            (c1, c2)
        } else {
            (c2, c1)
        };

        RatchetSessionState {
            root_key: root,
            send_chain_key,
            recv_chain_key,
            message_number: 0,
            previous_chain_length: 0,
            ratchet_public_key: vec![],
        }
    }

    /// Serialize to a base64-encoded JSON blob.
    pub fn encode(&self) -> Result<String, CryptoError> {
        let bytes = serde_json::to_vec(self)
            .map_err(|e| CryptoError::Serialization(e.to_string()))?;
        Ok(B64.encode(bytes))
    }

    /// Deserialize from a base64-encoded JSON blob.
    pub fn decode(s: &str) -> Result<Self, CryptoError> {
        let bytes = B64
            .decode(s)
            .map_err(|_| CryptoError::InvalidEncoding("session state".to_string()))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| CryptoError::InvalidInput(format!("session decode: {e}")))
    }
}

pub fn hkdf_expand(secret: &[u8], label: &[u8]) -> [u8; 32] {
    use hkdf::Hkdf;
    use sha2::Sha256;
    let hk = Hkdf::<Sha256>::new(None, secret);
    let mut out = [0u8; 32];
    hk.expand(label, &mut out)
        .expect("HKDF expand to 32 bytes should always succeed");
    out
}
