#[cfg(test)]
mod tests {
    use super::super::*;

    fn fake_seal_data() -> SealData {
        SealData {
            rand: [3u8; 16],
            isvsvn: 7,
            cpusvn: [9u8; 16],
            attributes: Attributes::default(),
            miscselect: Miscselect::default(),
        }
    }

    #[test]
    fn sealed_blob_crypto_round_trips_with_supplied_key() {
        let key = [0x42u8; 32];
        let nonce = [0x24u8; 12];
        let plaintext = b"sealed enclave payload";

        let sealed = seal_with_key(key, fake_seal_data(), plaintext, nonce).expect("seal");
        let blob: SealedBlob = bincode::deserialize(&sealed).expect("deserialize sealed blob");

        assert_eq!(blob.nonce, nonce);
        assert_ne!(blob.ciphertext.as_slice(), plaintext);
        assert_eq!(
            unseal_with_key(key, &blob).expect("unseal"),
            plaintext.as_slice()
        );
    }

    #[test]
    fn sealed_blob_rejects_modified_ciphertext() {
        let key = [0x42u8; 32];
        let sealed = seal_with_key(
            key,
            fake_seal_data(),
            b"sealed enclave payload",
            [0x24u8; 12],
        )
        .expect("seal");
        let mut blob: SealedBlob = bincode::deserialize(&sealed).expect("deserialize sealed blob");

        *blob.ciphertext.last_mut().expect("ciphertext") ^= 1;

        assert!(matches!(
            unseal_with_key(key, &blob),
            Err(SealError::Crypto)
        ));
    }

    #[cfg(not(target_env = "sgx"))]
    #[test]
    fn public_seal_reports_unsupported_outside_sgx() {
        assert!(matches!(
            seal(ENC_KEY, b"payload"),
            Err(SealError::UnsupportedTarget)
        ));
    }

    #[cfg(target_env = "sgx")]
    #[test]
    fn public_seal_round_trips_inside_sgx() {
        let plaintext = b"payload sealed by SGX";
        let sealed = seal(ENC_KEY, plaintext).expect("seal");

        assert_eq!(unseal(ENC_KEY, &sealed).expect("unseal"), plaintext);
    }

    #[cfg(target_env = "sgx")]
    #[test]
    fn public_unseal_rejects_wrong_label_inside_sgx() {
        let sealed = seal(ENC_KEY, b"payload sealed by SGX").expect("seal");

        assert!(matches!(unseal(PRF_KEY, &sealed), Err(SealError::Crypto)));
    }

    #[cfg(target_env = "sgx")]
    #[test]
    fn public_unseal_rejects_changed_enclave_metadata_inside_sgx() {
        let sealed = seal(ENC_KEY, b"payload sealed by SGX").expect("seal");
        let mut blob: SealedBlob = bincode::deserialize(&sealed).expect("deserialize sealed blob");
        blob.seal_data.miscselect =
            Miscselect::from_bits_truncate(blob.seal_data.miscselect.bits() ^ 1);
        let sealed = bincode::serialize(&blob).expect("serialize sealed blob");

        assert!(matches!(
            unseal(ENC_KEY, &sealed),
            Err(SealError::InvalidEnclaveAttributes)
        ));
    }
}
