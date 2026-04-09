use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::error::CryptoError;
use crate::keys::{IdentityBundle, OneTimePreKey, SignedPreKey};

pub fn generate_identity_bundle() -> IdentityBundle {
    let mut rng = OsRng;
    let identity_private = StaticSecret::random_from_rng(&mut rng);
    let identity_public = PublicKey::from(&identity_private);

    let signing_key = SigningKey::generate(&mut rng);
    let signing_public = signing_key.verifying_key();

    IdentityBundle {
        identity_key_pair: crate::keys::KeyPair {
            public: STANDARD.encode(identity_public.as_bytes()),
            private: STANDARD.encode(identity_private.to_bytes()),
        },
        signing_key_pair: crate::keys::KeyPair {
            public: STANDARD.encode(signing_public.as_bytes()),
            private: STANDARD.encode(signing_key.to_bytes()),
        },
    }
}

pub fn generate_signed_prekey(
    identity_signing_private: &str,
    prekey_id: u32,
) -> Result<SignedPreKey, CryptoError> {
    let signing_private_raw = STANDARD
        .decode(identity_signing_private)
        .map_err(|_| CryptoError::invalid("invalid signing private key"))?;
    let signing_private_arr: [u8; 32] = signing_private_raw
        .try_into()
        .map_err(|_| CryptoError::invalid("signing private key length must be 32 bytes"))?;
    let signing_key = SigningKey::from_bytes(&signing_private_arr);

    let mut rng = OsRng;
    let prekey_private = StaticSecret::random_from_rng(&mut rng);
    let prekey_public = PublicKey::from(&prekey_private);
    let signature = signing_key.sign(prekey_public.as_bytes());

    Ok(SignedPreKey {
        id: prekey_id,
        public_key: STANDARD.encode(prekey_public.as_bytes()),
        signature: STANDARD.encode(signature.to_bytes()),
        private_key: STANDARD.encode(prekey_private.to_bytes()),
    })
}

pub fn generate_one_time_prekeys(start_id: u32, count: u32) -> Vec<OneTimePreKey> {
    let mut out = Vec::with_capacity(count as usize);
    let mut rng = OsRng;
    for i in 0..count {
        let private = StaticSecret::random_from_rng(&mut rng);
        let public = PublicKey::from(&private);
        out.push(OneTimePreKey {
            id: start_id + i,
            public_key: STANDARD.encode(public.as_bytes()),
            private_key: STANDARD.encode(private.to_bytes()),
        });
    }
    out
}

pub fn verify_signature(
    signing_public: &str,
    message: &[u8],
    signature: &str,
) -> Result<bool, CryptoError> {
    let signing_public_raw = STANDARD
        .decode(signing_public)
        .map_err(|_| CryptoError::invalid("invalid signing public key"))?;
    let signing_public_arr: [u8; 32] = signing_public_raw
        .try_into()
        .map_err(|_| CryptoError::invalid("signing public key length must be 32 bytes"))?;

    let signature_raw = STANDARD
        .decode(signature)
        .map_err(|_| CryptoError::invalid("invalid signature"))?;
    let signature_arr: [u8; 64] = signature_raw
        .try_into()
        .map_err(|_| CryptoError::invalid("signature length must be 64 bytes"))?;

    let verifying_key = VerifyingKey::from_bytes(&signing_public_arr)
        .map_err(|_| CryptoError::invalid("invalid signing public key bytes"))?;
    let signature = ed25519_dalek::Signature::from_bytes(&signature_arr);

    Ok(verifying_key.verify(message, &signature).is_ok())
}
