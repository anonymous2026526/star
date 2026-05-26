use std::{io, sync::Mutex};

use bloomfilter::Bloom;

use crate::core::{FilterBox, bytes::Long};

const BLOOM_SEED: [u8; 32] = [0u8; 32];

pub struct BloomFilterBox {
    inner: Mutex<Bloom<Long>>,
}

impl BloomFilterBox {
    pub fn new(bit_len: usize, hash_functions: u32) -> Self {
        let bit_len = bit_len.max(8);
        let hash_functions = hash_functions.max(1);
        let bitmap_size = (bit_len + 7) / 8;
        let items =
            ((bit_len as f64 * std::f64::consts::LN_2) / hash_functions as f64).ceil() as usize;
        let items = items.max(1);
        let inner = Bloom::new_with_seed(bitmap_size, items, &BLOOM_SEED)
            .expect("bloomfilter init should succeed");
        Self {
            inner: Mutex::new(inner),
        }
    }

    pub fn with_rate(expected_items: usize, false_positive_rate: f64) -> Self {
        let n = expected_items.max(1);
        let p = false_positive_rate.clamp(1e-9, 0.5);
        let inner = Bloom::new_for_fp_rate_with_seed(n, p, &BLOOM_SEED)
            .expect("bloomfilter init should succeed");
        Self {
            inner: Mutex::new(inner),
        }
    }
}

impl FilterBox for BloomFilterBox {
    fn from_entries(tokens: &[Long]) -> io::Result<Self> {
        let filter = BloomFilterBox::with_rate(tokens.len(), 0.01);

        for t in tokens {
            filter.insert(&t)?;
        }

        Ok(filter)
    }

    fn insert(&self, token: &Long) -> io::Result<()> {
        self.inner
            .lock()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "bloom filter lock poisoned"))?
            .set(token);
        Ok(())
    }

    fn test(&self, token: &Long) -> bool {
        self.inner
            .lock()
            .map(|inner| !inner.check(token))
            .unwrap_or(false)
    }

    fn test_and_insert(&self, token: &Long) -> io::Result<bool> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "bloom filter lock poisoned"))?;
        let is_new = !inner.check(token);
        if is_new {
            inner.set(token);
        }
        Ok(is_new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_changes_test_result() {
        let mut filter = BloomFilterBox::new(8, 1);
        let token: Long = [7u8; 32];
        assert!(filter.test(&token));
        filter.insert(&token).unwrap();
        assert!(!filter.test(&token));
    }

    #[test]
    fn with_rate_clamps_inputs() {
        let mut filter = BloomFilterBox::with_rate(0, 1.0);
        let token: Long = [9u8; 32];
        assert!(filter.test(&token));
        filter.insert(&token).unwrap();
        assert!(!filter.test(&token));
    }
}
