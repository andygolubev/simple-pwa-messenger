use messenger_crypto::{
    generate_identity, generate_one_time_prekeys, generate_signed_prekey, ratchet_decrypt,
    ratchet_encrypt, x3dh_initiate, x3dh_respond, IdentityBundle,
};

#[test]
fn first_message_round_trip_via_x3dh_and_ratchet() {
    let alice = generate_identity().expect("alice");
    let bob = generate_identity().expect("bob");
    let bob_spk = generate_signed_prekey(&bob.identity_private_key, 42).expect("spk");
    let bob_opk = generate_one_time_prekeys(100, 1).expect("opk");

    let init = x3dh_initiate(
        &alice.identity_private_key,
        &bob.identity_public_key,
        &bob.signing_public_key,
        &bob_spk.public_key,
        &bob_spk.signature,
        bob_opk.first().cloned(),
    )
    .expect("init");
    let opk_private = bob_opk.first().expect("private opk").private_key.clone();

    let resp = x3dh_respond(
        &bob.identity_private_key,
        &bob_spk.private_key,
        Some(opk_private),
        &alice.identity_public_key,
        &init.ephemeral_public,
    )
    .expect("respond");

    let encrypted = ratchet_encrypt(&init.session_state, "hello encrypted world").expect("enc");
    let decrypted = ratchet_decrypt(
        &resp.session_state,
        &encrypted.message.header,
        &encrypted.message.ciphertext,
    )
    .expect("dec");
    assert_eq!(decrypted.plaintext, "hello encrypted world");
}

#[test]
fn identity_bundle_helper_contains_expected_fields() {
    let bundle = IdentityBundle::generate().expect("bundle");
    assert!(!bundle.identity_private_key.is_empty());
    assert!(!bundle.identity_public_key.is_empty());
    assert!(!bundle.signing_private_key.is_empty());
    assert!(!bundle.signing_public_key.is_empty());
}
