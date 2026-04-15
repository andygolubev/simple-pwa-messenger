use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::error::CryptoError;
use crate::identity;

/// Flat identity bundle returned by generate_identity() and IdentityBundle::generate().
/// Used by the public Rust API and WASM bindings.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdentityBundle {
    #[serde(rename = "identityPrivateKey")]
    pub identity_private_key: String,
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: String,
    #[serde(rename = "signingPrivateKey")]
    pub signing_private_key: String,
    #[serde(rename = "signingPublicKey")]
    pub signing_public_key: String,
}

impl IdentityBundle {
    pub fn generate() -> Result<Self, CryptoError> {
        identity::generate_identity_internal()
    }
}

/// A signed prekey with both public and private halves.
#[derive(Clone, Debug, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct SignedPreKey {
    pub id: u32,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub signature: String,
    #[serde(rename = "privateKey")]
    pub private_key: String,
}

/// A one-time prekey with both public and private halves.
#[derive(Clone, Debug, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct OneTimePreKey {
    pub id: u32,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    #[serde(rename = "privateKey")]
    pub private_key: String,
}

/// Result of x3dh_initiate: session state + X3DH handshake metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct X3dhInitiateResult {
    /// Base64-encoded serialized RatchetSessionState for the initiator.
    #[serde(rename = "sessionState")]
    pub session_state: String,
    /// Base64-encoded ephemeral X25519 public key sent to the responder.
    #[serde(rename = "ephemeralPublic")]
    pub ephemeral_public: String,
    /// ID of the one-time prekey consumed, if any.
    #[serde(rename = "usedOneTimePreKeyId")]
    pub used_one_time_pre_key_id: Option<u32>,
}

/// Result of x3dh_respond: session state for the responder.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct X3dhRespondResult {
    /// Base64-encoded serialized RatchetSessionState for the responder.
    #[serde(rename = "sessionState")]
    pub session_state: String,
}

/// Native X3DH initiate result that also exposes the raw root key (for tests).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct X3dhNativeInitiateResult {
    #[serde(rename = "sessionState")]
    pub session_state: String,
    #[serde(rename = "ephemeralPublic")]
    pub ephemeral_public: String,
    #[serde(rename = "usedOneTimePreKeyId")]
    pub used_one_time_pre_key_id: Option<u32>,
    /// Base64-encoded raw root key (for test assertions).
    #[serde(rename = "rootKey")]
    pub root_key: String,
}

/// Native X3DH respond result that also exposes the raw root key (for tests).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct X3dhNativeRespondResult {
    #[serde(rename = "sessionState")]
    pub session_state: String,
    /// Base64-encoded raw root key (for test assertions).
    #[serde(rename = "rootKey")]
    pub root_key: String,
}

/// Ratchet message (header + ciphertext, both base64-encoded).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RatchetMessage {
    pub header: String,
    pub ciphertext: String,
}

/// Result of ratchet_encrypt.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RatchetEncryptResult {
    #[serde(rename = "updatedSessionState")]
    pub updated_session_state: String,
    pub message: RatchetMessage,
}

/// Result of ratchet_decrypt.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RatchetDecryptResult {
    #[serde(rename = "updatedSessionState")]
    pub updated_session_state: String,
    pub plaintext: String,
}

/// Key pair used in NativeIdentityBundle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyPair {
    #[serde(rename = "publicKey")]
    pub public_key: String,
    #[serde(rename = "privateKey")]
    pub private_key: String,
}

/// Full identity bundle used by native (non-WASM) APIs, with methods for
/// generating prekeys using this identity's signing key.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeIdentityBundle {
    pub identity: KeyPair,
    pub signing: KeyPair,
}

impl NativeIdentityBundle {
    pub fn generate_signed_prekey(&self, prekey_id: u32) -> Result<SignedPreKey, CryptoError> {
        identity::generate_signed_prekey_internal(&self.signing.private_key, prekey_id)
    }

    pub fn generate_one_time_prekeys(
        &self,
        start_id: u32,
        count: u32,
    ) -> Result<Vec<OneTimePreKey>, CryptoError> {
        identity::generate_one_time_prekeys_internal(start_id, count)
    }
}
