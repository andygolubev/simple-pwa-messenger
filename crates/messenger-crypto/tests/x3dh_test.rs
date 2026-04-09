use messenger_crypto::{generate_identity_native, x3dh_initiate_native, x3dh_respond_native};

#[test]
fn x3dh_initiator_and_responder_derive_same_root_key() {
    let alice = generate_identity_native().expect("alice identity");
    let bob = generate_identity_native().expect("bob identity");
    let bob_signed = bob.generate_signed_prekey(1).expect("signed prekey");
    let bob_one_time = bob.generate_one_time_prekeys(10, 1).expect("opk");

    let initiated = x3dh_initiate_native(
        &alice.identity.private_key,
        &bob.identity.public_key,
        &bob_signed.public_key,
        &bob_signed.signature,
        Some(&bob.signing.public_key),
        Some(bob_one_time[0].public_key.clone()),
        Some(bob_one_time[0].id),
    )
    .expect("initiate");

    let responded = x3dh_respond_native(
        &bob.identity.private_key,
        &bob_signed.private_key,
        Some(bob_one_time[0].private_key.clone()),
        &alice.identity.public_key,
        &initiated.ephemeral_public,
    )
    .expect("respond");

    assert_eq!(initiated.root_key, responded.root_key);
}
