use alloc::vec::Vec;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use hpke::{
    aead::ChaCha20Poly1305 as HpkeChaCha20Poly1305,
    kdf::HkdfSha256,
    kem::{Kem as _, X25519HkdfSha256},
    setup_receiver, Deserializable, HpkeError, OpModeR, Serializable,
};
use rand::{CryptoRng, RngCore};

// HPKE receiver living inside the enclave.
pub struct SecureChannelServer {
    pub sk: <X25519HkdfSha256 as hpke::kem::Kem>::PrivateKey,
    pub pk: <X25519HkdfSha256 as hpke::kem::Kem>::PublicKey,
}

impl SecureChannelServer {
    /// Restore a receiver from an existing 32-byte X25519 secret key.
    pub fn new(sk_bytes: [u8; 32]) -> Result<Self, HpkeError> {
        let sk = <X25519HkdfSha256 as hpke::kem::Kem>::PrivateKey::from_bytes(&sk_bytes)?;
        let pk = X25519HkdfSha256::sk_to_pk(&sk);
        Ok(Self { sk, pk })
    }

    /// Generate a fresh receiver keypair using curve25519-dalek.
    pub fn random<R>(rng: &mut R) -> Self
    where
        R: CryptoRng + RngCore,
    {
        let (sk, pk) = X25519HkdfSha256::gen_keypair(rng);

        Self { sk, pk }
    }

    /// Public key for advertising to senders.
    pub fn public_key(&self) -> [u8; 32] {
        let bytes = self.pk.to_bytes();
        let bytes_ref: &[u8] = bytes.as_ref();
        bytes_ref.try_into().expect("public key must be 32 bytes")
    }

    /// Decrypt an HPKE ciphertext (X25519 + ChaCha20-Poly1305).
    pub fn receive_hpke(
        &self,
        enc: &[u8],
        info: &[u8],
        aad: &[u8],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, HpkeError> {
        let enc = <X25519HkdfSha256 as hpke::kem::Kem>::EncappedKey::from_bytes(enc)?;
        let mut ctx = setup_receiver::<HpkeChaCha20Poly1305, HkdfSha256, X25519HkdfSha256>(
            &OpModeR::Base,
            &self.sk,
            &enc,
            info,
        )?;

        ctx.open(ciphertext, aad)
    }

    pub fn respond_cipher<R>(&self, response: &[u8], cipher: &[u8], rng: &mut R) -> Vec<u8>
    where
        R: RngCore,
    {
        let Some((aad, info, enc, ciphertext)) = split_hpke_envelope(cipher) else {
            return Vec::new();
        };

        #[cfg(feature = "timecop")]
        timecop::poison(ciphertext);

        let key = match self.receive_hpke(enc, info, aad, ciphertext) {
            Ok(key) => key,
            Err(_) => return Vec::new(),
        };

        #[cfg(feature = "timecop")]
        timecop::poison(key.as_slice());

        let mut nonce_bytes = [0u8; 12];
        rng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let aead = match ChaCha20Poly1305::new_from_slice(&key) {
            Ok(aead) => aead,
            Err(_) => return Vec::new(),
        };
        let ciphertext = match aead.encrypt(nonce, response) {
            Ok(ciphertext) => ciphertext,
            Err(_) => return Vec::new(),
        };

        let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        result
    }
}

pub(crate) fn split_hpke_envelope(cipher: &[u8]) -> Option<(&[u8], &[u8], &[u8], &[u8])> {
    const AAD_LEN: usize = 32;
    const INFO_LEN: usize = 32;
    const ENC_LEN: usize = 32;

    let min_len = AAD_LEN + INFO_LEN + ENC_LEN;
    if cipher.len() < min_len {
        return None;
    }

    let aad = &cipher[..AAD_LEN];
    let info = &cipher[AAD_LEN..AAD_LEN + INFO_LEN];
    let enc = &cipher[AAD_LEN + INFO_LEN..AAD_LEN + INFO_LEN + ENC_LEN];
    let ciphertext = &cipher[AAD_LEN + INFO_LEN + ENC_LEN..];

    Some((aad, info, enc, ciphertext))
}
