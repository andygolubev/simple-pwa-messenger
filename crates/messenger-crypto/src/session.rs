use crate::double_ratchet::{
    decrypt_message, encrypt_message, MessageEnvelope, RatchetDecryptOutput, RatchetSession,
};
use crate::error::CryptoError;
use crate::keys::PreKeyBundle;
use crate::x3dh::{x3dh_initiate_internal, x3dh_respond_internal, X3dhInitialResult};

pub fn create_session_from_bundle(
    my_identity_private_b64: &str,
    their_bundle: &PreKeyBundle,
) -> Result<X3dhInitialResult, CryptoError> {
    x3dh_initiate_internal(
        my_identity_private_b64,
        &their_bundle.identity_public_key,
        &their_bundle.signed_prekey.public_key,
        &their_bundle.signed_prekey.signature,
        their_bundle.one_time_prekey.as_ref().map(|opk| opk.public_key.clone()),
        their_bundle.one_time_prekey.as_ref().map(|opk| opk.id),
    )
}

pub fn accept_initial_message(
    my_identity_private_b64: &str,
    my_signed_prekey_private_b64: &str,
    my_one_time_prekey_private_b64: Option<&str>,
    their_identity_public_b64: &str,
    their_ephemeral_public_b64: &str,
) -> Result<RatchetSession, CryptoError> {
    x3dh_respond_internal(
        my_identity_private_b64,
        my_signed_prekey_private_b64,
        my_one_time_prekey_private_b64,
        their_identity_public_b64,
        their_ephemeral_public_b64,
    )
}

pub fn send_message(
    session: &RatchetSession,
    plaintext: &str,
) -> Result<(RatchetSession, MessageEnvelope), CryptoError> {
    encrypt_message(session, plaintext)
}

pub fn receive_message(
    session: &RatchetSession,
    header: &str,
    ciphertext: &str,
) -> Result<RatchetDecryptOutput, CryptoError> {
    decrypt_message(session, header, ciphertext)
}
