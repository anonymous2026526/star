use constant_time_utils::time;
use hpke::Serializable;
use rand::{rngs::OsRng, RngCore};
use star_core::client::User;
use star_core::secure_channel::{client::SecureChannelClient, server::SecureChannelServer};
use star_core::serv::core::token;
use star_core::serv::enclave::Enclave;

const period: u64 = 1;

// #[cfg(feature = "timecop")]
fn main() {
    // End-to-end demo: issue a credential, request a token over HPKE, and inspect the token.
    let mut rng: OsRng = OsRng;

    // Bootstrap enclave secrets.
    let mut token_sk = [0u8; 32];
    let mut hpke_sk = [0u8; 32];
    rng.fill_bytes(&mut token_sk);
    rng.fill_bytes(&mut hpke_sk);

    timecop::poison(hpke_sk.as_slice());
    timecop::poison(token_sk.as_slice());

    let max_tokens_per_credential = 3u32;
    let enclave = Enclave::new(max_tokens_per_credential, token_sk, hpke_sk)
        .expect("failed to initialize enclave");

    // User setup and credential issuance.
    let pk = enclave.public_key();

    timecop::unpoison(pk.as_slice());
    let sender = SecureChannelClient::new(pk).expect("failed to initialize sender");
    let mut user = User::new(max_tokens_per_credential as u64, sender);
    let (cipher, ck) = user.request_credential(&mut rng);
    let credential_bytes: Vec<u8> = enclave.issue_cred(&cipher, &mut rng);

    timecop::unpoison(credential_bytes.as_slice());
    user.receive_credential(&credential_bytes, &ck);

    let envelope = user
        .request_auth(1, period, &mut rng)
        .expect("failed to build token request");
    let token = enclave
        .issue_token(&envelope, period.to_be_bytes())
        .expect("token issuance failed");
    timecop::unpoison(token.as_slice());

    fail1();
    fail2();

    println!("done");
}

use hpke::{
    aead::ChaCha20Poly1305,
    kdf::HkdfSha384,
    kem::{Kem as KemTrait, X25519HkdfSha256},
    Deserializable, HpkeError, OpModeR,
};

fn fail1() {
    type Aead = ChaCha20Poly1305;
    type Kdf = HkdfSha384;
    type Kem = X25519HkdfSha256;

    let sk_bytes = [0u8; 32];
    timecop::poison(sk_bytes.as_slice());

    let sk = <X25519HkdfSha256 as hpke::kem::Kem>::PrivateKey::from_bytes(&sk_bytes).unwrap();

    let plain = [0u8; 32];
    let bad_encapped_key =
        <Kem as KemTrait>::EncappedKey::from_bytes(&plain).expect("bad enc parse failed");

    let info = b"ct-test";

    let res = hpke::setup_receiver::<Aead, Kdf, Kem>(&OpModeR::Base, &sk, &bad_encapped_key, info);
    assert!(matches!(res, Err(HpkeError::DecapError)));
}

fn fail2() {
    let mut rng = OsRng;
    let mut hpke_sk = [0u8; 32];
    rng.fill_bytes(&mut hpke_sk);

    timecop::poison(hpke_sk.as_slice());
    let receiver = SecureChannelServer::new(hpke_sk).unwrap();

    let receiver_pk = receiver.public_key();
    timecop::unpoison(receiver_pk.as_slice());

    let sender = SecureChannelClient::new(receiver_pk).expect("sender init failed");
    let (envelope, _shared_key) = sender
        .send_shared_key(&mut rng)
        .expect("send shared key failed");

    let mut envelope = envelope.clone();
    const AAD_LEN: usize = 32;
    const INFO_LEN: usize = 32;
    const ENC_LEN: usize = 32;
    let min_len = AAD_LEN + INFO_LEN + ENC_LEN;
    assert!(envelope.len() >= min_len, "HPKE envelope too short");

    envelope[min_len - 1] = !envelope[min_len - 1];

    let aad = &envelope[..AAD_LEN];
    let info = &envelope[AAD_LEN..AAD_LEN + INFO_LEN];
    let enc = &envelope[AAD_LEN + INFO_LEN..AAD_LEN + INFO_LEN + ENC_LEN];
    let ciphertext = &envelope[AAD_LEN + INFO_LEN + ENC_LEN..];

    // this is target
    let decrypted_key = receiver.receive_hpke(enc, info, aad, ciphertext);
    assert!(matches!(decrypted_key, Err(hpke::HpkeError::OpenError)));
}
