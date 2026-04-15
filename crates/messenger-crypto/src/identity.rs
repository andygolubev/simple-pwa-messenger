use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand_core::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::error::CryptoError;
use crate::keys::{IdentityBundle, NativeIdentityBundle, KeyPair, OneTimePreKey, SignedPreKey};

pub fn generate_identity_internal() -> Result<IdentityBundle, CryptoError> {
    let identity_private = StaticSecret::random_from_rng(OsRng);
    let identity_public = PublicKey::from(&identity_private);

    let signing_key = SigningKey::generate(&mut OsRng);
    let signing_public = signing_key.verifying_key();

    Ok(IdentityBundle {
        identity_private_key: B64.encode(identity_private.to_bytes()),
        identity_public_key: B64.encode(identity_public.as_bytes()),
        signing_private_key: B64.encode(signing_key.to_bytes()),
        signing_public_key: B64.encode(signing_public.as_bytes()),
    })
}

pub fn generate_identity_native_internal() -> Result<NativeIdentityBundle, CryptoError> {
    let identity_private = StaticSecret::random_from_rng(OsRng);
    let identity_public = PublicKey::from(&identity_private);

    let signing_key = SigningKey::generate(&mut OsRng);
    let signing_public = signing_key.verifying_key();

    Ok(NativeIdentityBundle {
        identity: KeyPair {
            public_key: B64.encode(identity_public.as_bytes()),
            private_key: B64.encode(identity_private.to_bytes()),
        },
        signing: KeyPair {
            public_key: B64.encode(signing_public.as_bytes()),
            private_key: B64.encode(signing_key.to_bytes()),
        },
    })
}

/// Generate a signed prekey using the Ed25519 signing private key.
pub fn generate_signed_prekey_internal(
    signing_private_b64: &str,
    prekey_id: u32,
) -> Result<SignedPreKey, CryptoError> {
    let raw = B64
        .decode(signing_private_b64)
        .map_err(|_| CryptoError::InvalidBase64("signing_private"))?;
    let arr: [u8; 32] = raw
        .try_into()
        .map_err(|_| CryptoError::InvalidLength("signing_private"))?;
    let signing_key = SigningKey::from_bytes(&arr);

    let prekey_private = StaticSecret::random_from_rng(OsRng);
    let prekey_public = PublicKey::from(&prekey_private);
    let signature = signing_key.sign(prekey_public.as_bytes());

    Ok(SignedPreKey {
        id: prekey_id,
        public_key: B64.encode(prekey_public.as_bytes()),
        signature: B64.encode(signature.to_bytes()),
        private_key: B64.encode(prekey_private.to_bytes()),
    })
}

/// Generate a batch of one-time prekeys (X25519 DH pairs).
pub fn generate_one_time_prekeys_internal(
    start_id: u32,
    count: u32,
) -> Result<Vec<OneTimePreKey>, CryptoError> {
    let mut out = Vec::with_capacity(count as usize);
    for i in 0..count {
        let private = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&private);
        out.push(OneTimePreKey {
            id: start_id + i,
            public_key: B64.encode(public.as_bytes()),
            private_key: B64.encode(private.to_bytes()),
        });
    }
    Ok(out)
}

/// Decode an Ed25519 verifying key from base64.
pub fn decode_signing_public_key(b64: &str) -> Result<VerifyingKey, CryptoError> {
    let raw = B64
        .decode(b64)
        .map_err(|_| CryptoError::InvalidBase64("signing_public_key"))?;
    let arr: [u8; 32] = raw
        .try_into()
        .map_err(|_| CryptoError::InvalidLength("signing_public_key"))?;
    VerifyingKey::from_bytes(&arr)
        .map_err(|_| CryptoError::InvalidKeyMaterial("signing_public_key"))
}

/// Decode an Ed25519 signature from base64.
pub fn decode_signature(b64: &str) -> Result<ed25519_dalek::Signature, CryptoError> {
    let raw = B64
        .decode(b64)
        .map_err(|_| CryptoError::InvalidBase64("signature"))?;
    let arr: [u8; 64] = raw
        .try_into()
        .map_err(|_| CryptoError::InvalidLength("signature"))?;
    Ok(ed25519_dalek::Signature::from_bytes(&arr))
}

/// Verify an Ed25519 signature over a message using the signing public key.
pub fn verify_signed_prekey_signature(
    signing_public_b64: &str,
    prekey_public_b64: &str,
    signature_b64: &str,
) -> Result<(), CryptoError> {
    let verifying_key = decode_signing_public_key(signing_public_b64)?;
    let signature = decode_signature(signature_b64)?;
    let prekey_bytes = B64
        .decode(prekey_public_b64)
        .map_err(|_| CryptoError::InvalidBase64("signed_prekey_public"))?;
    verifying_key
        .verify_strict(&prekey_bytes, &signature)
        .map_err(|_| CryptoError::SignatureVerificationFailed)
}
