use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicPrivateKeyPair {
    pub public: String,
    pub private: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdentityBundle {
    #[serde(rename = "identityKeyPair")]
    pub identity_key_pair: PublicPrivateKeyPair,
    #[serde(rename = "signingKeyPair")]
    pub signing_key_pair: PublicPrivateKeyPair,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedPreKey {
    pub id: u32,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub signature: String,
    #[serde(rename = "privateKey")]
    pub private_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct OneTimePreKeyPrivate {
    pub id: u32,
    #[serde(rename = "privateKey")]
    pub private_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OneTimePreKeyPublic {
    pub id: u32,
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionEnvelope {
    #[serde(rename = "sessionState")]
    pub session_state: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RatchetMessage {
    pub header: String,
    pub ciphertext: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptResult {
    #[serde(rename = "updatedSessionState")]
    pub updated_session_state: String,
    pub message: RatchetMessage,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecryptResult {
    #[serde(rename = "updatedSessionState")]
    pub updated_session_state: String,
    pub plaintext: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct X3dhInitiateResult {
    #[serde(rename = "sessionState")]
    pub session_state: String,
    #[serde(rename = "ephemeralPublic")]
    pub ephemeral_public: String,
    #[serde(rename = "usedOneTimePreKeyId")]
    pub used_one_time_pre_key_id: Option<u32>,
}

