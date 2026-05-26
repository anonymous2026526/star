use crate::secure_channel::server::{split_hpke_envelope, SecureChannelServer};
use crate::serv::core::bytes::{Long, Short};
use crate::serv::core::cred::Credential;
use crate::serv::core::token::Token;

use alloc::vec::Vec;
use hpke::HpkeError;
use rand::RngCore;

#[cfg(feature = "timecop")]
use timecop;

// HPKE receiver living inside the enclave.
pub struct Enclave {
    cipher_receiver: SecureChannelServer,
    max_tickets: Short,
    sk: Long,
}

impl Enclave {
    /// Restore a receiver from an existing 32-byte X25519 secret key.
    pub fn new(max_tickets: u32, sk_bytes: Long, cipher_receiver: Long) -> Result<Self, HpkeError> {
        let cipher_receiver = SecureChannelServer::new(cipher_receiver)?;

        let max_tickets: Short = (max_tickets as u64).to_be_bytes();
        Ok(Self {
            cipher_receiver,
            max_tickets,
            sk: sk_bytes,
        })
    }

    /// Generate a fresh receiver keypair using curve25519-dalek.
    pub fn default<R>(rng: &mut R) -> Self
    where
        R: RngCore,
    {
        let mut cipher_receiver = [0u8; 32];
        rng.fill_bytes(&mut cipher_receiver);

        let mut sk = [0u8; 32];
        rng.fill_bytes(&mut sk);

        Self::new(0, sk, cipher_receiver).expect("failed to create enclave")
    }

    /// Public key for advertising to senders.
    pub fn public_key(&self) -> [u8; 32] {
        self.cipher_receiver.public_key()
    }

    pub fn issue_token(&self, cipher: &[u8], current_period: Short) -> Result<Vec<u8>, u8> {
        let (aad, info, enc, ciphertext) = split_hpke_envelope(cipher).ok_or(1)?;
        let plain = self
            .cipher_receiver
            .receive_hpke(enc, info, aad, ciphertext)
            .map_err(|_| 1)?;

        let result = self.issue_token_plain(&plain, current_period);

        Ok(result)
    }

    /// Issue a token from an already decrypted request (credential 64 bytes + count 8 bytes + period 8 bytes).
    /// This is mainly for examples/tests; production callers should use `issue_token` with HPKE-protected payloads.
    pub fn issue_token_plain(&self, request: &[u8], current_period: Short) -> Vec<u8> {
        #[cfg(feature = "timecop")]
        timecop::poison(request);

        if request.len() < 80 {
            return alloc::vec![0u8];
        }
        let cred = Credential::from_bytes(&request[0..64]);
        let count: Short = request[64..72].try_into().expect("fixed-length slice");
        let period: Short = request[72..80].try_into().expect("fixed-length slice");

        if !cred.is_valid(period, count, current_period, self.max_tickets, self.sk) {
            return alloc::vec![0u8];
        }
        let token = Token::random(cred.uid, period, count);
        let result = token.to_bytes();

        return result;
    }

    pub fn issue_cred<R>(&self, cipher: &[u8], rng: &mut R) -> Vec<u8>
    where
        R: RngCore,
    {
        let cred = self.issue_cred_plain(rng);

        #[cfg(feature = "timecop")]
        timecop::poison(cred.as_slice());

        // The HPKE envelope is aad(32) || info(32) || enc(32) || ciphertext.
        // respond_cipher expects the full envelope to recover the shared key.
        let result = self.cipher_receiver.respond_cipher(&cred, cipher, rng);

        result
    }

    pub fn issue_cred_plain<R>(&self, rng: &mut R) -> Vec<u8>
    where
        R: RngCore,
    {
        #[cfg(feature = "timecop")]
        timecop::poison(self.sk.as_slice());
        let cred = Credential::random(self.sk, rng);

        #[cfg(feature = "timecop")]
        timecop::poison(cred.uid.as_slice());

        let result = cred.to_bytes();

        return result;
    }
}
