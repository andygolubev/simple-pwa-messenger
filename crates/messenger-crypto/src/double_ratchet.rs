use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::error::CryptoError;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetState {
    pub root_key: String,
    pub send_chain_key: String,
    pub recv_chain_key: String,
    pub send_count: u32,
    pub recv_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetMessage {
    pub header: String,
    pub ciphertext: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptResult {
    pub updated_session_state: String,
    pub message: RatchetMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptResult {
    pub updated_session_state: String,
    pub plaintext: String,
}

pub fn derive_initial_state(shared_secret: &[u8], is_initiator: bool) -> RatchetState {
    let mut root = hkdf_expand(shared_secret, b"root");
    let mut c1 = hkdf_expand(&root, b"chain:1");
    let mut c2 = hkdf_expand(&root, b"chain:2");

    if !is_initiator {
        std::mem::swap(&mut c1, &mut c2);
    }

    // Roll root once to reduce direct linkage to X3DH output.
    root = hkdf_expand(&root, b"root:next");

    RatchetState {
        root_key: B64.encode(root),
        send_chain_key: B64.encode(c1),
        recv_chain_key: B64.encode(c2),
        send_count: 0,
        recv_count: 0,
    }
}

pub fn encrypt(state_b64: &str, plaintext: &str) -> Result<EncryptResult, CryptoError> {
    let mut state: RatchetState = decode_state(state_b64)?;
    let chain_key = B64
        .decode(&state.send_chain_key)
        .map_err(|_| CryptoError::InvalidEncoding("send chain key".into()))?;

    let message_key = derive_message_key(&chain_key)?;
    let next_chain = next_chain_key(&chain_key)?;

    let nonce = derive_nonce(&state.send_count);
    let cipher = Aes256Gcm::new_from_slice(&message_key)
        .map_err(|_| CryptoError::InvalidInput("invalid AES key length".into()))?;
    let encrypted = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())
        .map_err(|_| CryptoError::EncryptionFailed)?;

    let header = Header {
        pn: state.recv_count,
        n: state.send_count,
        rk: state.root_key.clone(),
    };

    state.send_chain_key = B64.encode(next_chain);
    state.send_count = state.send_count.saturating_add(1);

    Ok(EncryptResult {
        updated_session_state: encode_state(&state),
        message: RatchetMessage {
            header: B64.encode(
                serde_json::to_vec(&header)
                    .map_err(|_| CryptoError::Serialization("header".into()))?,
            ),
            ciphertext: B64.encode(encrypted),
        },
    })
}

pub fn decrypt(state_b64: &str, header_b64: &str, ciphertext_b64: &str) -> Result<DecryptResult, CryptoError> {
    let mut state: RatchetState = decode_state(state_b64)?;

    let header_bytes = B64
        .decode(header_b64)
        .map_err(|_| CryptoError::InvalidEncoding("header".into()))?;
    let header: Header = serde_json::from_slice(&header_bytes)
        .map_err(|_| CryptoError::InvalidInput("invalid header".into()))?;

    // Minimal ratchet-step signal: if root key moved, sync it.
    if header.rk != state.root_key {
        state.root_key = header.rk;
    }

    let chain_key = B64
        .decode(&state.recv_chain_key)
        .map_err(|_| CryptoError::InvalidEncoding("recv chain key".into()))?;
    let message_key = derive_message_key(&chain_key)?;
    let next_chain = next_chain_key(&chain_key)?;

    let nonce = derive_nonce(&header.n);
    let cipher = Aes256Gcm::new_from_slice(&message_key)
        .map_err(|_| CryptoError::InvalidInput("invalid AES key length".into()))?;
    let ciphertext = B64
        .decode(ciphertext_b64)
        .map_err(|_| CryptoError::InvalidEncoding("ciphertext".into()))?;

    let plaintext_bytes = cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| CryptoError::DecryptionFailed)?;

    state.recv_chain_key = B64.encode(next_chain);
    state.recv_count = state.recv_count.max(header.n.saturating_add(1));

    let plaintext = String::from_utf8(plaintext_bytes).map_err(|_| CryptoError::InvalidInput("plaintext is not UTF-8".into()))?;
    Ok(DecryptResult {
        updated_session_state: encode_state(&state),
        plaintext,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Header {
    pn: u32,
    n: u32,
    rk: String,
}

fn encode_state(state: &RatchetState) -> String {
    B64.encode(serde_json::to_vec(state).unwrap_or_default())
}

fn decode_state(state_b64: &str) -> Result<RatchetState, CryptoError> {
    let bytes = B64
        .decode(state_b64)
        .map_err(|_| CryptoError::InvalidEncoding("session state".into()))?;
    serde_json::from_slice(&bytes).map_err(|_| CryptoError::InvalidInput("invalid session state".into()))
}

fn derive_message_key(chain_key: &[u8]) -> Result<[u8; 32], CryptoError> {
    let mut mac = HmacSha256::new_from_slice(chain_key)
        .map_err(|_| CryptoError::InvalidInput("invalid chain key".into()))?;
    mac.update(b"message_key");
    let out = mac.finalize().into_bytes();
    let mut key = [0u8; 32];
    key.copy_from_slice(&out[..32]);
    Ok(key)
}

fn next_chain_key(chain_key: &[u8]) -> Result<[u8; 32], CryptoError> {
    let mut mac = HmacSha256::new_from_slice(chain_key)
        .map_err(|_| CryptoError::InvalidInput("invalid chain key".into()))?;
    mac.update(b"chain_step");
    let out = mac.finalize().into_bytes();
    let mut next = [0u8; 32];
    next.copy_from_slice(&out[..32]);
    Ok(next)
}

fn hkdf_expand(secret: &[u8], label: &[u8]) -> [u8; 32] {
    use hkdf::Hkdf;
    let hk = Hkdf::<Sha256>::new(None, secret);
    let mut out = [0u8; 32];
    hk.expand(label, &mut out).expect("HKDF expand for 32 bytes must work");
    out
}

fn derive_nonce(counter: &u32) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[8..12].copy_from_slice(&counter.to_be_bytes());
    nonce
}
