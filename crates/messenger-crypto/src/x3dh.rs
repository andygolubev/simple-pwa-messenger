use base64::{engine::general_purpose::STANDARD, Engine as _};
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::error::{CryptoError, CryptoResult};
use crate::keys::X3dhBundle;

fn decode_public_key(base64_value: &str, field: &'static str) -> CryptoResult<PublicKey> {
    let bytes = STANDARD
        .decode(base64_value)
        .map_err(|_| CryptoError::InvalidBase64(field))?;
    let key: [u8; 32] = bytes
        .try_into()
        .map_err(|_| CryptoError::InvalidLength(field))?;
    Ok(PublicKey::from(key))
}

fn decode_private_key(base64_value: &str, field: &'static str) -> CryptoResult<StaticSecret> {
    let bytes = STANDARD
        .decode(base64_value)
        .map_err(|_| CryptoError::InvalidBase64(field))?;
    let key: [u8; 32] = bytes
        .try_into()
        .map_err(|_| CryptoError::InvalidLength(field))?;
    Ok(StaticSecret::from(key))
}

pub fn derive_master_secret_initiator(
    my_identity_private: &str,
    their_identity_public: &str,
    their_signed_prekey_public: &str,
    their_one_time_prekey_public: Option<&str>,
    my_ephemeral_private: &str,
) -> CryptoResult<[u8; 32]> {
    let ik_a = decode_private_key(my_identity_private, "my_identity_private")?;
    let ik_b = decode_public_key(their_identity_public, "their_identity_public")?;
    let spk_b = decode_public_key(their_signed_prekey_public, "their_signed_prekey")?;
    let ek_a = decode_private_key(my_ephemeral_private, "my_ephemeral_private")?;

    let dh1 = ik_a.diffie_hellman(&spk_b);
    let dh2 = ek_a.diffie_hellman(&ik_b);
    let dh3 = ek_a.diffie_hellman(&spk_b);

    let mut input = Vec::with_capacity(128);
    input.extend_from_slice(dh1.as_bytes());
    input.extend_from_slice(dh2.as_bytes());
    input.extend_from_slice(dh3.as_bytes());

    if let Some(opk_b64) = their_one_time_prekey_public {
        let opk_b = decode_public_key(opk_b64, "their_one_time_prekey")?;
        let dh4 = ek_a.diffie_hellman(&opk_b);
        input.extend_from_slice(dh4.as_bytes());
    }

    let hk = Hkdf::<Sha256>::new(None, &input);
    let mut output = [0u8; 32];
    hk.expand(b"messenger-x3dh-v1", &mut output)
        .map_err(|_| CryptoError::InvalidKeyMaterial("hkdf expand"))?;
    Ok(output)
}

pub fn derive_master_secret_responder(
    my_identity_private: &str,
    my_signed_prekey_private: &str,
    my_one_time_prekey_private: Option<&str>,
    their_identity_public: &str,
    their_ephemeral_public: &str,
) -> CryptoResult<[u8; 32]> {
    let ik_b = decode_private_key(my_identity_private, "my_identity_private")?;
    let spk_b = decode_private_key(my_signed_prekey_private, "my_signed_prekey_private")?;
    let ik_a = decode_public_key(their_identity_public, "their_identity_public")?;
    let ek_a = decode_public_key(their_ephemeral_public, "their_ephemeral_public")?;

    let dh1 = spk_b.diffie_hellman(&ik_a);
    let dh2 = ik_b.diffie_hellman(&ek_a);
    let dh3 = spk_b.diffie_hellman(&ek_a);

    let mut input = Vec::with_capacity(128);
    input.extend_from_slice(dh1.as_bytes());
    input.extend_from_slice(dh2.as_bytes());
    input.extend_from_slice(dh3.as_bytes());

    if let Some(opk_b64) = my_one_time_prekey_private {
        let opk_b = decode_private_key(opk_b64, "my_one_time_prekey_private")?;
        let dh4 = opk_b.diffie_hellman(&ek_a);
        input.extend_from_slice(dh4.as_bytes());
    }

    let hk = Hkdf::<Sha256>::new(None, &input);
    let mut output = [0u8; 32];
    hk.expand(b"messenger-x3dh-v1", &mut output)
        .map_err(|_| CryptoError::InvalidKeyMaterial("hkdf expand"))?;
    Ok(output)
}

pub fn verify_signed_prekey(bundle: &X3dhBundle) -> CryptoResult<()> {
    let verify_key = crate::identity::decode_signing_public_key(&bundle.signing_public_key)?;
    let signature = crate::identity::decode_signature(&bundle.signed_prekey_signature)?;
    let signed_prekey_bytes = STANDARD
        .decode(&bundle.signed_prekey)
        .map_err(|_| CryptoError::InvalidBase64("signed_prekey"))?;
    verify_key
        .verify_strict(&signed_prekey_bytes, &signature)
        .map_err(|_| CryptoError::SignatureVerificationFailed)
}
