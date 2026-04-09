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

