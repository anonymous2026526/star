use crate::secure_channel::{client::SecureChannelClient, server::SecureChannelServer};

#[test]
fn test_all() {
    use rand::rngs::OsRng;

    let mut rng = OsRng;
    let receiver = SecureChannelServer::random(&mut rng);
    let receiver_pk = receiver.public_key();

    let sender = SecureChannelClient::new(receiver_pk).expect("sender init failed");
    let (envelope, shared_key) = sender
        .send_shared_key(&mut rng)
        .expect("send shared key failed");

    const AAD_LEN: usize = 32;
    const INFO_LEN: usize = 32;
    const ENC_LEN: usize = 32;
    let min_len = AAD_LEN + INFO_LEN + ENC_LEN;
    assert!(envelope.len() >= min_len, "HPKE envelope too short");

    let aad = &envelope[..AAD_LEN];
    let info = &envelope[AAD_LEN..AAD_LEN + INFO_LEN];
    let enc = &envelope[AAD_LEN + INFO_LEN..AAD_LEN + INFO_LEN + ENC_LEN];
    let ciphertext = &envelope[AAD_LEN + INFO_LEN + ENC_LEN..];

    let decrypted_key = receiver
        .receive_hpke(enc, info, aad, ciphertext)
        .expect("receiver failed to decrypt shared key");
    assert_eq!(decrypted_key.as_slice(), &shared_key);

    let response = b"hello-from-receiver";
    let response_cipher = receiver.respond_cipher(response, &envelope, &mut rng);
    let response_plaintext = sender
        .decrypt_with_shared_key(&response_cipher, &shared_key)
        .expect("sender failed to decrypt receiver response");
    assert_eq!(response_plaintext.as_slice(), response);
}
