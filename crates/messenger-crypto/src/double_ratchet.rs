use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::error::CryptoError;
use crate::session::RatchetSessionState;

type HmacSha256 = Hmac<Sha256>;

/// Ratchet message header stored alongside each ciphertext.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetHeader {
    /// Message number in the current sending chain.
    pub n: u32,
    /// Length of the previous sending chain (for out-of-order recovery).
    pub pn: u32,
}

/// Output of ratchet_encrypt_internal.
pub struct EncryptedMessage {
    /// Base64-encoded JSON RatchetHeader.
    pub header: String,
    /// Base64-encoded AES-256-GCM ciphertext.
    pub ciphertext: String,
}

/// Encrypt a plaintext message and advance the send chain.
/// Mutates `state` in place (increments message_number, rotates send_chain_key).
pub fn ratchet_encrypt_internal(
    state: &mut RatchetSessionState,
    plaintext: &str,
) -> Result<EncryptedMessage, CryptoError> {
    let message_key = derive_message_key(&state.send_chain_key)?;
    state.send_chain_key = next_chain_key(&state.send_chain_key)?;

    let header = RatchetHeader {
        n: state.message_number,
        pn: state.previous_chain_length,
    };
    state.message_number = state.message_number.saturating_add(1);

    let nonce = derive_nonce(header.n);
    let cipher = Aes256Gcm::new_from_slice(&message_key)
        .map_err(|_| CryptoError::InvalidKeyMaterial("message_key for AES-GCM"))?;
    let encrypted = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())
        .map_err(|_| CryptoError::EncryptionFailed)?;

    let header_json = serde_json::to_vec(&header)
        .map_err(|e| CryptoError::Serialization(e.to_string()))?;

    Ok(EncryptedMessage {
        header: B64.encode(header_json),
        ciphertext: B64.encode(encrypted),
    })
}

/// Decrypt a message and advance the receive chain.
/// Mutates `state` in place (increments message_number, rotates recv_chain_key).
pub fn ratchet_decrypt_internal(
    state: &mut RatchetSessionState,
    header_b64: &str,
    ciphertext_b64: &str,
) -> Result<String, CryptoError> {
    let header_bytes = B64
        .decode(header_b64)
        .map_err(|_| CryptoError::InvalidEncoding("header".to_string()))?;
    let header: RatchetHeader = serde_json::from_slice(&header_bytes)
        .map_err(|e| CryptoError::InvalidInput(format!("header decode: {e}")))?;

    let message_key = derive_message_key(&state.recv_chain_key)?;
    state.recv_chain_key = next_chain_key(&state.recv_chain_key)?;
    state.message_number = state.message_number.max(header.n.saturating_add(1));

    let nonce = derive_nonce(header.n);
    let cipher = Aes256Gcm::new_from_slice(&message_key)
        .map_err(|_| CryptoError::InvalidKeyMaterial("message_key for AES-GCM"))?;

    let ciphertext = B64
        .decode(ciphertext_b64)
        .map_err(|_| CryptoError::InvalidEncoding("ciphertext".to_string()))?;

    let plaintext_bytes = cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| CryptoError::DecryptionFailed)?;

    String::from_utf8(plaintext_bytes)
        .map_err(|_| CryptoError::InvalidInput("plaintext is not valid UTF-8".to_string()))
}

fn derive_message_key(chain_key: &[u8; 32]) -> Result<[u8; 32], CryptoError> {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(chain_key)
        .map_err(|_| CryptoError::InvalidInput("invalid chain key for HMAC".to_string()))?;
    mac.update(b"messenger:message_key");
    let out = mac.finalize().into_bytes();
    let mut key = [0u8; 32];
    key.copy_from_slice(&out[..32]);
    Ok(key)
}

fn next_chain_key(chain_key: &[u8; 32]) -> Result<[u8; 32], CryptoError> {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(chain_key)
        .map_err(|_| CryptoError::InvalidInput("invalid chain key for HMAC".to_string()))?;
    mac.update(b"messenger:chain_step");
    let out = mac.finalize().into_bytes();
    let mut next = [0u8; 32];
    next.copy_from_slice(&out[..32]);
    Ok(next)
}

fn derive_nonce(counter: u32) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[8..12].copy_from_slice(&counter.to_be_bytes());
    nonce
}
