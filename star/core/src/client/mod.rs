use alloc::vec::Vec;
use rand::{CryptoRng, RngCore};

use crate::{
    secure_channel,
    serv::core::{bytes::Short, cred::Credential, token::Token},
};

/// Client-side holder for credentials and token requests.
pub struct User {
    pub credential: Option<Credential>,
    pub client: secure_channel::client::SecureChannelClient,
    pub max: u64,
}

impl User {
    /// Create a user allowed to make an unlimited number of token requests.
    /// Create a user capped at `max` token requests for a credential.
    pub fn new(max: u64, client: secure_channel::client::SecureChannelClient) -> Self {
        Self {
            credential: None,
            client,
            max,
        }
    }

    pub fn request_credential<R>(&self, rng: &mut R) -> (Vec<u8>, [u8; 32])
    where
        R: CryptoRng + RngCore,
    {
        self.client
            .send_shared_key(rng)
            .expect("send shared key error")
    }

    pub fn receive_credential(
        &mut self,
        cipher: &[u8],
        shared_key: &[u8; 32],
    ) -> Option<Credential> {
        let cred = self
            .client
            .decrypt_with_shared_key(cipher, shared_key)
            .expect("failed to decrypto cred");
        self.receive_credential_plain(&cred);
        self.credential
    }

    /// Store a freshly issued credential.
    pub fn receive_credential_plain(&mut self, credential_bytes: &[u8]) -> Option<Credential> {
        let credential: Credential = Credential::from_bytes(credential_bytes);
        self.credential = Some(credential);
        self.credential
    }

    /// Present the credential by building a token request (credential + count + period).
    pub fn request_auth_plain(&mut self, counter: u64, period: u64) -> Vec<u8> {
        if counter >= self.max {
            panic!("exceeded max token requests for this credential");
        }

        let credential = self
            .credential
            .as_ref()
            .expect("credential not received yet");

        let count: Short = counter.to_be_bytes();
        let period: Short = period.to_be_bytes();
        let credential_bytes = credential.to_bytes();

        let mut token_request =
            Vec::with_capacity(credential_bytes.len() + count.len() + period.len());
        token_request.extend_from_slice(&credential_bytes);
        token_request.extend_from_slice(&count);
        token_request.extend_from_slice(&period);
        token_request
    }

    pub fn route(&mut self, counter: u64, period: u64, max: u32) -> u32 {
        let count: Short = counter.to_be_bytes();
        let period: Short = period.to_be_bytes();
        let token = Token::random(self.credential.unwrap().uid, period, count);

        let route = token.route(max);

        return route;
    }

    /// Present the credential over HPKE to the enclave.
    /// Returns an encrypted payload ready for `Enclave::issue_token`.
    pub fn request_auth<R>(
        &mut self,
        counter: u64,
        period: u64,
        rng: &mut R,
    ) -> Result<Vec<u8>, hpke::HpkeError>
    where
        R: CryptoRng + RngCore,
    {
        let token_request = self.request_auth_plain(counter, period);
        let cipher = self.client.send_hpke(&token_request, rng)?;

        Ok(cipher)
    }
}
