use messenger_crypto::{
    generate_identity, generate_one_time_prekeys, generate_signed_prekey, ratchet_decrypt,
    ratchet_encrypt, x3dh_initiate_session, x3dh_respond_session, IdentityBundle,
};

#[test]
fn first_message_round_trip_via_x3dh_and_ratchet() {
    let alice = generate_identity().expect("alice");
    let bob = generate_identity().expect("bob");

    // generate_signed_prekey takes the *signing* private key (Ed25519)
    let bob_spk = generate_signed_prekey(&bob.signing_private_key, 42).expect("spk");
    let bob_opk = generate_one_time_prekeys(100, 1).expect("opk");

    let opk_for_initiate = bob_opk.first().cloned();
    let opk_private = bob_opk.first().expect("private opk").private_key.clone();

    let init = x3dh_initiate_session(
        &alice.identity_private_key,
        &bob.identity_public_key,
        &bob.signing_public_key,
        &bob_spk.public_key,
        &bob_spk.signature,
        opk_for_initiate,
    )
    .expect("initiate");

    let resp = x3dh_respond_session(
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

#[test]
fn multiple_ratchet_messages_round_trip() {
    let alice = generate_identity().expect("alice");
    let bob = generate_identity().expect("bob");
    let bob_spk = generate_signed_prekey(&bob.signing_private_key, 1).expect("spk");

    let init = x3dh_initiate_session(
        &alice.identity_private_key,
        &bob.identity_public_key,
        &bob.signing_public_key,
        &bob_spk.public_key,
        &bob_spk.signature,
        None,
    )
    .expect("initiate");

    let resp = x3dh_respond_session(
        &bob.identity_private_key,
        &bob_spk.private_key,
        None,
        &alice.identity_public_key,
        &init.ephemeral_public,
    )
    .expect("respond");

    let messages = ["first", "second", "third"];
    let mut alice_state = init.session_state.clone();
    let mut bob_state = resp.session_state.clone();

    for msg in &messages {
        let enc = ratchet_encrypt(&alice_state, msg).expect("encrypt");
        alice_state = enc.updated_session_state.clone();

        let dec = ratchet_decrypt(&bob_state, &enc.message.header, &enc.message.ciphertext)
            .expect("decrypt");
        bob_state = dec.updated_session_state.clone();

        assert_eq!(&dec.plaintext, msg);
    }
}
