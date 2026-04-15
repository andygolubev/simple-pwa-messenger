pub mod double_ratchet;
pub mod error;
pub mod identity;
pub mod keys;
pub mod session;
pub mod x3dh;

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{Verifier, VerifyingKey};
use wasm_bindgen::prelude::*;

use crate::double_ratchet::{ratchet_decrypt_internal, ratchet_encrypt_internal};
use crate::error::CryptoError;
use crate::identity::{
    decode_signature, generate_identity_internal, generate_identity_native_internal,
    generate_one_time_prekeys_internal, generate_signed_prekey_internal,
};
use crate::keys::{
    OneTimePreKey, RatchetDecryptResult, RatchetEncryptResult, RatchetMessage,
    X3dhInitiateResult, X3dhNativeInitiateResult, X3dhNativeRespondResult, X3dhRespondResult,
};
use crate::session::RatchetSessionState;
use crate::x3dh::{
    x3dh_initiate, x3dh_initiate_native as x3dh_initiate_native_fn,
    x3dh_respond, x3dh_respond_native as x3dh_respond_native_fn,
};

// ─── Re-exports ──────────────────────────────────────────────────────────────

pub use crate::keys::{IdentityBundle, KeyPair, NativeIdentityBundle, SignedPreKey};

// ─── Public Rust API (non-WASM) ──────────────────────────────────────────────

/// Generate a new identity bundle (X25519 identity keys + Ed25519 signing keys).
pub fn generate_identity() -> Result<IdentityBundle, CryptoError> {
    generate_identity_internal()
}

/// Generate a native identity bundle with a `NativeIdentityBundle` type that
/// exposes `.identity` and `.signing` key pairs and helper methods.
pub fn generate_identity_native() -> Result<NativeIdentityBundle, CryptoError> {
    generate_identity_native_internal()
}

/// Generate a signed prekey. `signing_private_b64` must be the Ed25519 signing
/// private key (the `signingPrivateKey` field of an `IdentityBundle`).
pub fn generate_signed_prekey(
    signing_private_b64: &str,
    prekey_id: u32,
) -> Result<SignedPreKey, CryptoError> {
    generate_signed_prekey_internal(signing_private_b64, prekey_id)
}

/// Generate a batch of one-time prekeys.
pub fn generate_one_time_prekeys(
    start_id: u32,
    count: u32,
) -> Result<Vec<OneTimePreKey>, CryptoError> {
    generate_one_time_prekeys_internal(start_id, count)
}

/// Initiate an X3DH session as the sender.
pub fn x3dh_initiate_session(
    my_identity_private_b64: &str,
    their_identity_public_b64: &str,
    their_signing_public_b64: &str,
    their_signed_prekey_b64: &str,
    their_signed_prekey_signature_b64: &str,
    one_time_prekey: Option<OneTimePreKey>,
) -> Result<X3dhInitiateResult, CryptoError> {
    x3dh_initiate(
        my_identity_private_b64,
        their_identity_public_b64,
        their_signing_public_b64,
        their_signed_prekey_b64,
        their_signed_prekey_signature_b64,
        one_time_prekey,
    )
}

/// Respond to an X3DH initiation as the recipient.
pub fn x3dh_respond_session(
    my_identity_private_b64: &str,
    my_signed_prekey_private_b64: &str,
    my_one_time_prekey_private_b64: Option<String>,
    their_identity_public_b64: &str,
    their_ephemeral_public_b64: &str,
) -> Result<X3dhRespondResult, CryptoError> {
    x3dh_respond(
        my_identity_private_b64,
        my_signed_prekey_private_b64,
        my_one_time_prekey_private_b64.as_deref(),
        their_identity_public_b64,
        their_ephemeral_public_b64,
    )
}

/// Initiate an X3DH session — native variant that also returns the raw root key (for tests).
pub fn x3dh_initiate_native(
    my_identity_private: &str,
    their_identity_public: &str,
    their_signed_prekey: &str,
    their_signed_prekey_signature: &str,
    their_signing_public: Option<&str>,
    their_one_time_prekey_public: Option<String>,
    their_one_time_prekey_id: Option<u32>,
) -> Result<X3dhNativeInitiateResult, CryptoError> {
    x3dh_initiate_native_fn(
        my_identity_private,
        their_identity_public,
        their_signed_prekey,
        their_signed_prekey_signature,
        their_signing_public,
        their_one_time_prekey_public,
        their_one_time_prekey_id,
    )
}

/// Respond to an X3DH initiation — native variant that also returns the raw root key (for tests).
pub fn x3dh_respond_native(
    my_identity_private: &str,
    my_signed_prekey_private: &str,
    my_one_time_prekey_private: Option<String>,
    their_identity_public: &str,
    their_ephemeral_public: &str,
) -> Result<X3dhNativeRespondResult, CryptoError> {
    x3dh_respond_native_fn(
        my_identity_private,
        my_signed_prekey_private,
        my_one_time_prekey_private,
        their_identity_public,
        their_ephemeral_public,
    )
}

/// Encrypt a message using the Double Ratchet.
/// `session_state` is the base64-encoded serialized `RatchetSessionState`.
/// Returns an updated session state and the encrypted message.
pub fn ratchet_encrypt(
    session_state: &str,
    plaintext: &str,
) -> Result<RatchetEncryptResult, CryptoError> {
    let mut state = RatchetSessionState::decode(session_state)?;
    let msg = ratchet_encrypt_internal(&mut state, plaintext)?;
    let updated_session_state = state.encode()?;
    Ok(RatchetEncryptResult {
        updated_session_state,
        message: RatchetMessage {
            header: msg.header,
            ciphertext: msg.ciphertext,
        },
    })
}

/// Decrypt a message using the Double Ratchet.
/// Returns the plaintext and the updated serialized session state.
pub fn ratchet_decrypt(
    session_state: &str,
    header: &str,
    ciphertext: &str,
) -> Result<RatchetDecryptResult, CryptoError> {
    let mut state = RatchetSessionState::decode(session_state)?;
    let plaintext = ratchet_decrypt_internal(&mut state, header, ciphertext)?;
    let updated_session_state = state.encode()?;
    Ok(RatchetDecryptResult {
        updated_session_state,
        plaintext,
    })
}

// ─── WASM bindings ───────────────────────────────────────────────────────────

fn to_js<T: serde::Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

fn from_crypto_err(e: CryptoError) -> JsValue {
    JsValue::from(e)
}

#[wasm_bindgen(js_name = generateIdentity)]
pub fn wasm_generate_identity() -> Result<JsValue, JsValue> {
    generate_identity_internal()
        .map_err(from_crypto_err)
        .and_then(|v| to_js(&v))
}

#[wasm_bindgen(js_name = generateSignedPrekey)]
pub fn wasm_generate_signed_prekey(
    signing_private: &str,
    prekey_id: u32,
) -> Result<JsValue, JsValue> {
    generate_signed_prekey_internal(signing_private, prekey_id)
        .map_err(from_crypto_err)
        .and_then(|v| to_js(&v))
}

#[wasm_bindgen(js_name = generateOneTimePrekeys)]
pub fn wasm_generate_one_time_prekeys(start_id: u32, count: u32) -> Result<JsValue, JsValue> {
    generate_one_time_prekeys_internal(start_id, count)
        .map_err(from_crypto_err)
        .and_then(|v| to_js(&v))
}

#[wasm_bindgen(js_name = x3dhInitiate)]
pub fn wasm_x3dh_initiate(
    my_identity_private: &str,
    their_identity_public: &str,
    their_signing_public: &str,
    their_signed_prekey: &str,
    their_signed_prekey_signature: &str,
    their_one_time_prekey: Option<String>,
) -> Result<JsValue, JsValue> {
    let opk = their_one_time_prekey
        .as_deref()
        .map(|s| serde_json::from_str::<OneTimePreKey>(s))
        .transpose()
        .map_err(|e| JsValue::from_str(&format!("invalid one_time_prekey JSON: {e}")))?;

    x3dh_initiate(
        my_identity_private,
        their_identity_public,
        their_signing_public,
        their_signed_prekey,
        their_signed_prekey_signature,
        opk,
    )
    .map_err(from_crypto_err)
    .and_then(|v| to_js(&v))
}

#[wasm_bindgen(js_name = x3dhRespond)]
pub fn wasm_x3dh_respond(
    my_identity_private: &str,
    my_signed_prekey_private: &str,
    my_one_time_prekey_private: Option<String>,
    their_identity_public: &str,
    their_ephemeral_public: &str,
) -> Result<JsValue, JsValue> {
    x3dh_respond(
        my_identity_private,
        my_signed_prekey_private,
        my_one_time_prekey_private.as_deref(),
        their_identity_public,
        their_ephemeral_public,
    )
    .map_err(from_crypto_err)
    .and_then(|v| to_js(&v))
}

#[wasm_bindgen(js_name = ratchetEncrypt)]
pub fn wasm_ratchet_encrypt(session_state: &str, plaintext: &str) -> Result<JsValue, JsValue> {
    ratchet_encrypt(session_state, plaintext)
        .map_err(from_crypto_err)
        .and_then(|v| to_js(&v))
}

#[wasm_bindgen(js_name = ratchetDecrypt)]
pub fn wasm_ratchet_decrypt(
    session_state: &str,
    header: &str,
    ciphertext: &str,
) -> Result<JsValue, JsValue> {
    ratchet_decrypt(session_state, header, ciphertext)
        .map_err(from_crypto_err)
        .and_then(|v| to_js(&v))
}

#[wasm_bindgen(js_name = verifyIdentitySignature)]
pub fn wasm_verify_identity_signature(
    signing_public: &str,
    message: &str,
    signature: &str,
) -> bool {
    let public_key_bytes = match B64.decode(signing_public) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let Ok(arr): Result<[u8; 32], _> = public_key_bytes.try_into() else {
        return false;
    };
    let verifying_key = match VerifyingKey::from_bytes(&arr) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let sig = match decode_signature(signature) {
        Ok(s) => s,
        Err(_) => return false,
    };
    verifying_key.verify(message.as_bytes(), &sig).is_ok()
}
