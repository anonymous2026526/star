use alloc::vec::Vec;
use rand::RngCore;

use crate::serv::core::bytes::{Long, Short};

#[cfg(feature = "timecop")]
use timecop;

#[derive(Copy, Clone)]
pub struct Credential {
    pub uid: Long,
    pub code: Long,
}

impl Credential {
    pub fn new(uid: Long, code: Long) -> Self {
        Self { uid, code }
    }

    pub fn random<R>(sk: Long, rng: &mut R) -> Self
    where
        R: RngCore,
    {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        // random uid
        let mut uid: [u8; 32] = [0u8; 32];
        rng.fill_bytes(&mut uid);

        #[cfg(feature = "timecop")]
        timecop::poison(uid.as_slice());

        // calc code ← MAC(uid, sk)
        let mut mac = Hmac::<Sha256>::new_from_slice(&sk).expect("HMAC can take key of any size");
        mac.update(&uid);
        let code: Long = mac.finalize().into_bytes().into();

        Self { uid, code }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let uid: Long = bytes[0..32].try_into().expect("fixed-length slice");
        let code: Long = bytes[32..64].try_into().expect("fixed-length slice");

        Self { uid, code }
    }

    pub fn is_valid(
        &self,
        period: Short,
        count: Short,
        current_period: Short,
        max_count: Short,
        sk: Long,
    ) -> bool {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let is_current = constant_time_utils::cmp::ct_eq(&period, &current_period) as u8;
        let is_count_valid = constant_time_utils::cmp::ct_lt(&count, &max_count) as u8;
        // panic!("hello: {:?}\n{:?}\n{:?}", is_count_valid, count, max_count);

        // Recalculate code and compare
        let mut mac = Hmac::<Sha256>::new_from_slice(&sk).expect("HMAC can take key of any size");
        mac.update(&self.uid);
        let recalculated_code: Long = mac.finalize().into_bytes().into();
        let is_code_valid = constant_time_utils::cmp::ct_eq(&recalculated_code, &self.code) as u8;

        // panic!("hello: {}: {}: {}", is_current, is_count_valid, is_code_valid);

        (is_current & is_count_valid & is_code_valid) == 1
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.uid);
        bytes.extend_from_slice(&self.code);
        bytes
    }
}
