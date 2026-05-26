// core/src/bin/measure_auth_bytes.rs

use rand::{rngs::OsRng, RngCore};
use star_core::client::User;
use star_core::secure_channel::client::SecureChannelClient;
use star_core::serv::enclave::Enclave;

const PERIOD: u64 = 1;
const COUNTER: u64 = 0;

fn main() {
    let mut rng = OsRng;

    // Bootstrap enclave secrets.
    let mut token_sk = [0u8; 32];
    let mut hpke_sk = [0u8; 32];
    rng.fill_bytes(&mut token_sk);
    rng.fill_bytes(&mut hpke_sk);

    let max_tokens_per_credential = 3u32;

    let enclave = Enclave::new(max_tokens_per_credential, token_sk, hpke_sk)
        .expect("failed to initialize enclave");

    let sender =
        SecureChannelClient::new(enclave.public_key()).expect("failed to initialize sender");

    let mut user = User::new(max_tokens_per_credential as u64, sender);

    // Issue a credential first, because authentication requires a credential.
    let (credential_request, shared_key) = user.request_credential(&mut rng);
    let credential_response = enclave.issue_cred(&credential_request, &mut rng);

    user.receive_credential(&credential_response, &shared_key)
        .expect("failed to receive credential");

    // Plain authentication request:
    // credential(64) || count(8) || period(8) = 80 bytes
    let plain_auth_request = user.request_auth_plain(COUNTER, PERIOD);

    // Actual bytes sent by the user during authentication:
    // HPKE envelope: aad(32) || info(32) || enc(32) || ciphertext
    let encrypted_auth_request = user
        .request_auth(COUNTER, PERIOD, &mut rng)
        .expect("failed to build auth request");

    println!(
        "credential issuance request bytes: {}",
        credential_request.len()
    );
    println!(
        "credential issuance response bytes: {}",
        credential_response.len()
    );

    println!("auth plaintext bytes: {}", plain_auth_request.len());
    println!("auth sent bytes: {}", encrypted_auth_request.len());

    // Optional sanity check: make sure the generated request is accepted.
    let token = enclave
        .issue_token(&encrypted_auth_request, PERIOD.to_be_bytes())
        .expect("token issuance failed");

    println!("token response bytes: {}", token.len());
}
