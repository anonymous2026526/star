use rand::{rngs::OsRng, RngCore};
use star_core::client::User;
use star_core::secure_channel::client::SecureChannelClient;
use star_core::serv::enclave::Enclave;

const period: u64 = 1;

fn main() {
    // End-to-end demo: issue a credential, request a token over HPKE, and inspect the token.
    let mut rng = OsRng;

    // Bootstrap enclave secrets.
    let mut token_sk = [0u8; 32];
    let mut hpke_sk = [0u8; 32];
    rng.fill_bytes(&mut token_sk);
    rng.fill_bytes(&mut hpke_sk);

    let max_tokens_per_credential = 3u32;
    let enclave = Enclave::new(max_tokens_per_credential, token_sk, hpke_sk)
        .expect("failed to initialize enclave");

    // User setup and credential issuance.
    let sender =
        SecureChannelClient::new(enclave.public_key()).expect("failed to initialize sender");
    let mut user = User::new(max_tokens_per_credential as u64, sender);
    let (cipher, ck) = user.request_credential(&mut rng);
    let credential_bytes = enclave.issue_cred(&cipher, &mut rng);
    user.receive_credential(&credential_bytes, &ck);
    println!(
        "User received credential ({} bytes): {}",
        credential_bytes.len(),
        hex(&credential_bytes)
    );

    // User asks for tokens using HPKE-wrapped requests.
    let mut issue_for_period = |counter| {
        let envelope = user
            .request_auth(counter, period, &mut rng)
            .expect("failed to build token request");
        enclave
            .issue_token(&envelope, period.to_be_bytes())
            .expect("token issuance failed")
    };

    let token_bytes3 = issue_for_period(1);
    let token_bytes1 = issue_for_period(0);
    let token_bytes2 = issue_for_period(0);

    for (idx, token_bytes) in [
        token_bytes1.as_slice(),
        token_bytes2.as_slice(),
        token_bytes3.as_slice(),
    ]
    .iter()
    .enumerate()
    {
        println!(
            "Issued token #{} ({} bytes): {}",
            idx + 1,
            token_bytes.len(),
            hex(token_bytes)
        );
    }

    assert_eq!(token_bytes1, token_bytes2);
    assert_ne!(token_bytes1, token_bytes3);
}

/// Hex formatter to avoid pulling an extra dependency for the example.
fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}
