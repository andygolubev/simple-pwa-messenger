use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use hkdf::Hkdf;
use rand_core::OsRng;
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::error::CryptoError;
use crate::identity::verify_signed_prekey_signature;
use crate::keys::{
    OneTimePreKey, X3dhInitiateResult, X3dhNativeInitiateResult, X3dhNativeRespondResult,
    X3dhRespondResult,
};
use crate::session::RatchetSessionState;

fn decode_x25519_private(b64: &str, field: &'static str) -> Result<StaticSecret, CryptoError> {
    let bytes = B64.decode(b64).map_err(|_| CryptoError::InvalidBase64(field))?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| CryptoError::InvalidLength(field))?;
    Ok(StaticSecret::from(arr))
}

fn decode_x25519_public(b64: &str, field: &'static str) -> Result<PublicKey, CryptoError> {
    let bytes = B64.decode(b64).map_err(|_| CryptoError::InvalidBase64(field))?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| CryptoError::InvalidLength(field))?;
    Ok(PublicKey::from(arr))
}

/// Run the X3DH key agreement as the initiator.
/// Returns the 32-byte master secret derived from X3DH DH operations.
fn x3dh_initiator_secret(
    my_identity_private: &StaticSecret,
    their_identity_public: &PublicKey,
    their_signed_prekey: &PublicKey,
    their_one_time_prekey: Option<&PublicKey>,
    my_ephemeral_private: &StaticSecret,
) -> [u8; 32] {
    let dh1 = my_identity_private.diffie_hellman(their_signed_prekey);
    let dh2 = my_ephemeral_private.diffie_hellman(their_identity_public);
    let dh3 = my_ephemeral_private.diffie_hellman(their_signed_prekey);

    let mut ikm = Vec::with_capacity(128);
    ikm.extend_from_slice(dh1.as_bytes());
    ikm.extend_from_slice(dh2.as_bytes());
    ikm.extend_from_slice(dh3.as_bytes());

    if let Some(opk) = their_one_time_prekey {
        let dh4 = my_ephemeral_private.diffie_hellman(opk);
        ikm.extend_from_slice(dh4.as_bytes());
    }

    let hk = Hkdf::<Sha256>::new(None, &ikm);
    let mut out = [0u8; 32];
    hk.expand(b"messenger-x3dh-v1", &mut out)
        .expect("HKDF expand to 32 bytes should always succeed");
    out
}

/// Run the X3DH key agreement as the responder.
fn x3dh_responder_secret(
    my_identity_private: &StaticSecret,
    my_signed_prekey_private: &StaticSecret,
    my_one_time_prekey_private: Option<&StaticSecret>,
    their_identity_public: &PublicKey,
    their_ephemeral_public: &PublicKey,
) -> [u8; 32] {
    let dh1 = my_signed_prekey_private.diffie_hellman(their_identity_public);
    let dh2 = my_identity_private.diffie_hellman(their_ephemeral_public);
    let dh3 = my_signed_prekey_private.diffie_hellman(their_ephemeral_public);

    let mut ikm = Vec::with_capacity(128);
    ikm.extend_from_slice(dh1.as_bytes());
    ikm.extend_from_slice(dh2.as_bytes());
    ikm.extend_from_slice(dh3.as_bytes());

    if let Some(opk) = my_one_time_prekey_private {
        let dh4 = opk.diffie_hellman(their_ephemeral_public);
        ikm.extend_from_slice(dh4.as_bytes());
    }

    let hk = Hkdf::<Sha256>::new(None, &ikm);
    let mut out = [0u8; 32];
    hk.expand(b"messenger-x3dh-v1", &mut out)
        .expect("HKDF expand to 32 bytes should always succeed");
    out
}

/// Initiate an X3DH session. Returns a session result including serialized
/// RatchetSessionState for the initiator and the ephemeral public key to send.
pub fn x3dh_initiate(
    my_identity_private_b64: &str,
    their_identity_public_b64: &str,
    their_signing_public_b64: &str,
    their_signed_prekey_b64: &str,
    their_signed_prekey_signature_b64: &str,
    one_time_prekey: Option<OneTimePreKey>,
) -> Result<X3dhInitiateResult, CryptoError> {
    // Verify the signed prekey signature before proceeding (TOFU guard)
    verify_signed_prekey_signature(
        their_signing_public_b64,
        their_signed_prekey_b64,
        their_signed_prekey_signature_b64,
    )?;

    let my_identity = decode_x25519_private(my_identity_private_b64, "my_identity_private")?;
    let their_identity = decode_x25519_public(their_identity_public_b64, "their_identity_public")?;
    let their_spk = decode_x25519_public(their_signed_prekey_b64, "their_signed_prekey")?;

    let ephemeral_private = StaticSecret::random_from_rng(OsRng);
    let ephemeral_public = PublicKey::from(&ephemeral_private);

    let their_opk_pub = one_time_prekey
        .as_ref()
        .map(|k| decode_x25519_public(&k.public_key, "their_one_time_prekey"))
        .transpose()?;

    let master_secret = x3dh_initiator_secret(
        &my_identity,
        &their_identity,
        &their_spk,
        their_opk_pub.as_ref(),
        &ephemeral_private,
    );

    let session = RatchetSessionState::from_shared_secret(&master_secret, true);
    let session_state = session.encode()?;

    Ok(X3dhInitiateResult {
        session_state,
        ephemeral_public: B64.encode(ephemeral_public.as_bytes()),
        used_one_time_pre_key_id: one_time_prekey.map(|k| k.id),
    })
}

/// Respond to an X3DH initiation. Returns a session result including serialized
/// RatchetSessionState for the responder.
pub fn x3dh_respond(
    my_identity_private_b64: &str,
    my_signed_prekey_private_b64: &str,
    my_one_time_prekey_private_b64: Option<&str>,
    their_identity_public_b64: &str,
    their_ephemeral_public_b64: &str,
) -> Result<X3dhRespondResult, CryptoError> {
    let my_identity = decode_x25519_private(my_identity_private_b64, "my_identity_private")?;
    let my_spk = decode_x25519_private(my_signed_prekey_private_b64, "my_signed_prekey_private")?;
    let their_identity = decode_x25519_public(their_identity_public_b64, "their_identity_public")?;
    let their_ephemeral =
        decode_x25519_public(their_ephemeral_public_b64, "their_ephemeral_public")?;

    let my_opk = my_one_time_prekey_private_b64
        .map(|b64| decode_x25519_private(b64, "my_one_time_prekey_private"))
        .transpose()?;

    let master_secret = x3dh_responder_secret(
        &my_identity,
        &my_spk,
        my_opk.as_ref(),
        &their_identity,
        &their_ephemeral,
    );

    let session = RatchetSessionState::from_shared_secret(&master_secret, false);
    let session_state = session.encode()?;

    Ok(X3dhRespondResult { session_state })
}

/// Native initiate: returns root_key in addition to session state (for tests).
pub fn x3dh_initiate_native(
    my_identity_private_b64: &str,
    their_identity_public_b64: &str,
    their_signed_prekey_b64: &str,
    their_signed_prekey_signature_b64: &str,
    their_signing_public_b64: Option<&str>,
    their_one_time_prekey_public: Option<String>,
    their_one_time_prekey_id: Option<u32>,
) -> Result<X3dhNativeInitiateResult, CryptoError> {
    if let Some(spub) = their_signing_public_b64 {
        verify_signed_prekey_signature(
            spub,
            their_signed_prekey_b64,
            their_signed_prekey_signature_b64,
        )?;
    }

    let my_identity = decode_x25519_private(my_identity_private_b64, "my_identity_private")?;
    let their_identity = decode_x25519_public(their_identity_public_b64, "their_identity_public")?;
    let their_spk = decode_x25519_public(their_signed_prekey_b64, "their_signed_prekey")?;

    let ephemeral_private = StaticSecret::random_from_rng(OsRng);
    let ephemeral_public = PublicKey::from(&ephemeral_private);

    let their_opk = their_one_time_prekey_public
        .as_deref()
        .map(|b64| decode_x25519_public(b64, "their_one_time_prekey"))
        .transpose()?;

    let master_secret = x3dh_initiator_secret(
        &my_identity,
        &their_identity,
        &their_spk,
        their_opk.as_ref(),
        &ephemeral_private,
    );

    let session = RatchetSessionState::from_shared_secret(&master_secret, true);
    let session_state = session.encode()?;

    Ok(X3dhNativeInitiateResult {
        session_state,
        ephemeral_public: B64.encode(ephemeral_public.as_bytes()),
        used_one_time_pre_key_id: their_one_time_prekey_id,
        root_key: B64.encode(master_secret),
    })
}

/// Native respond: returns root_key in addition to session state (for tests).
pub fn x3dh_respond_native(
    my_identity_private_b64: &str,
    my_signed_prekey_private_b64: &str,
    my_one_time_prekey_private: Option<String>,
    their_identity_public_b64: &str,
    their_ephemeral_public_b64: &str,
) -> Result<X3dhNativeRespondResult, CryptoError> {
    let my_identity = decode_x25519_private(my_identity_private_b64, "my_identity_private")?;
    let my_spk = decode_x25519_private(my_signed_prekey_private_b64, "my_signed_prekey_private")?;
    let their_identity = decode_x25519_public(their_identity_public_b64, "their_identity_public")?;
    let their_ephemeral =
        decode_x25519_public(their_ephemeral_public_b64, "their_ephemeral_public")?;

    let my_opk = my_one_time_prekey_private
        .as_deref()
        .map(|b64| decode_x25519_private(b64, "my_one_time_prekey_private"))
        .transpose()?;

    let master_secret = x3dh_responder_secret(
        &my_identity,
        &my_spk,
        my_opk.as_ref(),
        &their_identity,
        &their_ephemeral,
    );

    let session = RatchetSessionState::from_shared_secret(&master_secret, false);
    let session_state = session.encode()?;

    Ok(X3dhNativeRespondResult {
        session_state,
        root_key: B64.encode(master_secret),
    })
}
