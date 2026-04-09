mod double_ratchet;
mod error;
mod identity;
mod keys;
mod session;
mod x3dh;

use base64::Engine;
use error::{CryptoError, CryptoResult};
use keys::{
    IdentityKeyPair, OneTimePreKeyPrivate, SignedPreKeyPrivate, SigningKeyPair, StoredSessionState,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use x3dh::{build_initiator_session, build_responder_session};

#[derive(Debug, Serialize)]
struct WasmError {
    error: String,
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|err| JsValue::from_str(&err.to_string()))
}

fn to_js_error(err: CryptoError) -> JsValue {
    let payload = WasmError {
        error: err.to_string(),
    };
    serde_wasm_bindgen::to_value(&payload).unwrap_or_else(|_| JsValue::from_str("crypto error"))
}

fn parse_json<T: for<'de> Deserialize<'de>>(raw: &str) -> CryptoResult<T> {
    serde_json::from_str(raw)
        .map_err(|err| CryptoError::InvalidInput(format!("Failed to parse JSON: {err}")))
}

fn encode_session(session: &StoredSessionState) -> CryptoResult<String> {
    let serialized = serde_json::to_vec(session)
        .map_err(|err| CryptoError::Serde(format!("Failed to serialize session: {err}")))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(serialized))
}

fn decode_session(raw: &str) -> CryptoResult<StoredSessionState> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(raw)
        .map_err(|err| CryptoError::InvalidInput(format!("Invalid session encoding: {err}")))?;
    serde_json::from_slice(&bytes)
        .map_err(|err| CryptoError::Serde(format!("Failed to decode session: {err}")))
}

fn encode_header(header: &keys::RatchetHeader) -> CryptoResult<String> {
    let serialized = serde_json::to_vec(header)
        .map_err(|err| CryptoError::Serde(format!("Failed to encode header: {err}")))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(serialized))
}

fn decode_header(raw: &str) -> CryptoResult<keys::RatchetHeader> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(raw)
        .map_err(|err| CryptoError::InvalidInput(format!("Invalid header encoding: {err}")))?;
    serde_json::from_slice(&bytes)
        .map_err(|err| CryptoError::Serde(format!("Failed to decode header: {err}")))
}

#[wasm_bindgen]
pub fn generate_identity() -> Result<JsValue, JsValue> {
    identity::generate_identity()
        .and_then(|bundle| to_js(&bundle).map_err(CryptoError::Wasm))
        .map_err(to_js_error)
}

#[wasm_bindgen]
pub fn generate_signed_prekey(
    identity_private: &str,
    signing_private: &str,
    prekey_id: u32,
) -> Result<JsValue, JsValue> {
    identity::generate_signed_prekey(identity_private, signing_private, prekey_id)
        .and_then(|value| to_js(&value).map_err(CryptoError::Wasm))
        .map_err(to_js_error)
}

#[wasm_bindgen]
pub fn generate_one_time_prekeys(start_id: u32, count: u32) -> Result<JsValue, JsValue> {
    identity::generate_one_time_prekeys(start_id, count)
        .and_then(|value| to_js(&value).map_err(CryptoError::Wasm))
        .map_err(to_js_error)
}

#[derive(Debug, Deserialize)]
struct X3dhInitiateInput {
    my_identity_private: String,
    their_identity_public: String,
    their_signed_prekey: String,
    their_signed_prekey_signature: String,
    their_signing_public: String,
    their_one_time_prekey: Option<String>,
}

#[wasm_bindgen]
pub fn x3dh_initiate(input_json: &str) -> Result<JsValue, JsValue> {
    let input = parse_json::<X3dhInitiateInput>(input_json).map_err(to_js_error)?;

    build_initiator_session(
        &input.my_identity_private,
        &input.their_identity_public,
        &input.their_signed_prekey,
        &input.their_signed_prekey_signature,
        &input.their_signing_public,
        input.their_one_time_prekey.as_deref(),
    )
    .and_then(|value| to_js(&value).map_err(CryptoError::Wasm))
    .map_err(to_js_error)
}

#[derive(Debug, Deserialize)]
struct X3dhRespondInput {
    my_identity_private: String,
    my_signed_prekey_private: String,
    my_one_time_prekey_private: Option<String>,
    their_identity_public: String,
    their_ephemeral_public: String,
}

#[wasm_bindgen]
pub fn x3dh_respond(input_json: &str) -> Result<JsValue, JsValue> {
    let input = parse_json::<X3dhRespondInput>(input_json).map_err(to_js_error)?;

    build_responder_session(
        &input.my_identity_private,
        &input.my_signed_prekey_private,
        input.my_one_time_prekey_private.as_deref(),
        &input.their_identity_public,
        &input.their_ephemeral_public,
    )
    .and_then(|value| to_js(&value).map_err(CryptoError::Wasm))
    .map_err(to_js_error)
}

#[derive(Debug, Serialize)]
struct RatchetMessagePayload {
    header: String,
    ciphertext: String,
}

#[derive(Debug, Serialize)]
struct RatchetEncryptResponse {
    updatedSessionState: String,
    message: RatchetMessagePayload,
}

#[wasm_bindgen]
pub fn ratchet_encrypt(session_state: &str, plaintext: &str) -> Result<JsValue, JsValue> {
    let mut session = decode_session(session_state).map_err(to_js_error)?;
    let (header, ciphertext) =
        session::encrypt_message(&mut session, plaintext.as_bytes()).map_err(to_js_error)?;

    let response = RatchetEncryptResponse {
        updatedSessionState: encode_session(&session).map_err(to_js_error)?,
        message: RatchetMessagePayload {
            header: encode_header(&header).map_err(to_js_error)?,
            ciphertext: base64::engine::general_purpose::STANDARD.encode(ciphertext),
        },
    };

    to_js(&response).map_err(|err| to_js_error(CryptoError::Wasm(err)))
}

#[derive(Debug, Serialize)]
struct RatchetDecryptResponse {
    updatedSessionState: String,
    plaintext: String,
}

#[wasm_bindgen]
pub fn ratchet_decrypt(
    session_state: &str,
    header: &str,
    ciphertext: &str,
) -> Result<JsValue, JsValue> {
    let mut session = decode_session(session_state).map_err(to_js_error)?;
    let header = decode_header(header).map_err(to_js_error)?;
    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(ciphertext)
        .map_err(|err| to_js_error(CryptoError::InvalidInput(err.to_string())))?;

    let plaintext =
        session::decrypt_message(&mut session, &header, &ciphertext).map_err(to_js_error)?;
    let response = RatchetDecryptResponse {
        updatedSessionState: encode_session(&session).map_err(to_js_error)?,
        plaintext: String::from_utf8(plaintext)
            .map_err(|err| to_js_error(CryptoError::Serde(err.to_string())))?,
    };

    to_js(&response).map_err(|err| to_js_error(CryptoError::Wasm(err)))
}

#[wasm_bindgen]
pub fn verify_identity_signature(signing_public: &str, message: &str, signature: &str) -> bool {
    identity::verify_identity_signature(signing_public, message, signature).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use keys::{SignedPreKeyPublic, X3dhInitiateResponse, X3dhRespondResponse};

    #[test]
    fn session_round_trip_encode_decode() {
        let sender_identity = IdentityKeyPair::generate();
        let sender_signing = SigningKeyPair::generate();
        let receiver_identity = IdentityKeyPair::generate();
        let receiver_signing = SigningKeyPair::generate();

        let signed_prekey = SignedPreKeyPrivate::generate(42);
        let signed_public = SignedPreKeyPublic::from_private(&signed_prekey, &receiver_signing).unwrap();
        let one_time = OneTimePreKeyPrivate::generate(7);

        let initiate = build_initiator_session(
            &sender_identity.private_key,
            &receiver_identity.public_key,
            &signed_public.public_key,
            &signed_public.signature,
            &receiver_signing.public_key,
            Some(&one_time.public_key),
        )
        .unwrap();

        let initiate: X3dhInitiateResponse = initiate;
        let session = decode_session(&initiate.session_state).unwrap();
        let encoded = encode_session(&session).unwrap();
        assert_eq!(encoded, initiate.session_state);

        let respond = build_responder_session(
            &receiver_identity.private_key,
            &signed_prekey.private_key,
            Some(&one_time.private_key),
            &sender_identity.public_key,
            &initiate.ephemeral_public,
        )
        .unwrap();

        let respond: X3dhRespondResponse = respond;
        assert!(!respond.session_state.is_empty());
        assert!(!sender_signing.public_key.is_empty());
    }
}
pub mod double_ratchet;
pub mod error;
pub mod identity;
pub mod keys;
pub mod session;
pub mod util;
pub mod x3dh;

use wasm_bindgen::prelude::*;

use crate::double_ratchet::{ratchet_decrypt_internal, ratchet_encrypt_internal};
use crate::identity::{
    generate_identity_internal, generate_one_time_prekeys_internal, generate_signed_prekey_internal,
};
use crate::util::{decode_base64, decode_fixed_32};
use crate::x3dh::{x3dh_initiate_internal, x3dh_respond_internal, OneTimePreKeyRef};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

fn to_js_value<T: serde::Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|err| JsValue::from_str(&err.to_string()))
}

#[wasm_bindgen]
pub fn generate_identity() -> Result<JsValue, JsValue> {
    let output = generate_identity_internal().map_err(JsValue::from)?;
    to_js_value(&output)
}

#[wasm_bindgen]
pub fn generate_signed_prekey(signing_private: &str, prekey_id: u32) -> Result<JsValue, JsValue> {
    let output = generate_signed_prekey_internal(signing_private, prekey_id).map_err(JsValue::from)?;
    to_js_value(&output)
}

#[wasm_bindgen]
pub fn generate_one_time_prekeys(start_id: u32, count: u32) -> Result<JsValue, JsValue> {
    let output = generate_one_time_prekeys_internal(start_id, count).map_err(JsValue::from)?;
    to_js_value(&output)
}

#[wasm_bindgen]
pub fn x3dh_initiate(
    my_identity_private: &str,
    their_identity_public: &str,
    their_signing_public: &str,
    their_signed_prekey: &str,
    their_signed_prekey_signature: &str,
    their_one_time_prekey: Option<String>,
) -> Result<JsValue, JsValue> {
    let one_time_prekey = their_one_time_prekey
        .as_deref()
        .map(serde_json::from_str::<OneTimePreKeyRef>)
        .transpose()
        .map_err(|err| JsValue::from_str(&format!("invalid one-time prekey payload: {err}")))?;

    let output = x3dh_initiate_internal(
        my_identity_private,
        their_identity_public,
        their_signing_public,
        their_signed_prekey,
        their_signed_prekey_signature,
        one_time_prekey,
    )
    .map_err(JsValue::from)?;

    to_js_value(&output)
}

#[wasm_bindgen]
pub fn x3dh_respond(
    my_identity_private: &str,
    my_signed_prekey_private: &str,
    my_one_time_prekey_private: Option<String>,
    their_identity_public: &str,
    their_ephemeral_public: &str,
) -> Result<JsValue, JsValue> {
    let output = x3dh_respond_internal(
        my_identity_private,
        my_signed_prekey_private,
        my_one_time_prekey_private.as_deref(),
        their_identity_public,
        their_ephemeral_public,
    )
    .map_err(JsValue::from)?;

    to_js_value(&output)
}

#[wasm_bindgen]
pub fn ratchet_encrypt(session_state: &str, plaintext: &str) -> Result<JsValue, JsValue> {
    let output = ratchet_encrypt_internal(session_state, plaintext).map_err(JsValue::from)?;
    to_js_value(&output)
}

#[wasm_bindgen]
pub fn ratchet_decrypt(session_state: &str, header: &str, ciphertext: &str) -> Result<JsValue, JsValue> {
    let output = ratchet_decrypt_internal(session_state, header, ciphertext).map_err(JsValue::from)?;
    to_js_value(&output)
}

#[wasm_bindgen]
pub fn verify_identity_signature(signing_public: &str, message: &str, signature: &str) -> bool {
    let public_key_bytes = match decode_fixed_32(signing_public, "signing_public") {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    let signature_bytes = match decode_base64(signature, "signature") {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let verifying_key = match VerifyingKey::from_bytes(&public_key_bytes) {
        Ok(value) => value,
        Err(_) => return false,
    };
    let signature = match Signature::from_slice(&signature_bytes) {
        Ok(value) => value,
        Err(_) => return false,
    };

    verifying_key.verify(message.as_bytes(), &signature).is_ok()
}
