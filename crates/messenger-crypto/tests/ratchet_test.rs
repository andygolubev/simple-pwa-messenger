use messenger_crypto::double_ratchet::{ratchet_decrypt_internal, ratchet_encrypt_internal};
use messenger_crypto::session::RatchetSessionState;

#[test]
fn ratchet_round_trip_and_state_progression() {
    let mut sender = RatchetSessionState {
        root_key: [7; 32],
        send_chain_key: [8; 32],
        recv_chain_key: [8; 32],
        message_number: 0,
        previous_chain_length: 0,
        ratchet_public_key: vec![9; 32],
    };
    let mut receiver = sender.clone();

    let encrypted = ratchet_encrypt_internal(&mut sender, "hello").expect("encrypt");
    assert_eq!(sender.message_number, 1);

    let decrypted =
        ratchet_decrypt_internal(&mut receiver, &encrypted.header, &encrypted.ciphertext)
            .expect("decrypt");
    assert_eq!(decrypted, "hello");
    assert_eq!(receiver.message_number, 1);
}

#[test]
fn ratchet_sequential_messages() {
    let shared = [42u8; 32];
    let mut alice = RatchetSessionState::from_shared_secret(&shared, true);
    let mut bob = RatchetSessionState::from_shared_secret(&shared, false);

    for i in 0..5u32 {
        let plaintext = format!("message {i}");
        let enc = ratchet_encrypt_internal(&mut alice, &plaintext).expect("encrypt");
        let dec = ratchet_decrypt_internal(&mut bob, &enc.header, &enc.ciphertext)
            .expect("decrypt");
        assert_eq!(dec, plaintext);
    }

    assert_eq!(alice.message_number, 5);
    assert_eq!(bob.message_number, 5);
}

#[test]
fn ratchet_session_encode_decode_round_trip() {
    let state = RatchetSessionState {
        root_key: [1; 32],
        send_chain_key: [2; 32],
        recv_chain_key: [3; 32],
        message_number: 7,
        previous_chain_length: 3,
        ratchet_public_key: vec![4; 32],
    };

    let encoded = state.encode().expect("encode");
    let decoded = RatchetSessionState::decode(&encoded).expect("decode");

    assert_eq!(decoded.root_key, state.root_key);
    assert_eq!(decoded.send_chain_key, state.send_chain_key);
    assert_eq!(decoded.recv_chain_key, state.recv_chain_key);
    assert_eq!(decoded.message_number, state.message_number);
    assert_eq!(decoded.previous_chain_length, state.previous_chain_length);
}
