#[cfg(test)]
mod tests {
    use super::super::*;

    fn att_key_id(algorithm: u32) -> Vec<u8> {
        let mut id = vec![0u8; ATT_KEY_ALGORITHM_OFFSET + 4];
        id[ATT_KEY_ALGORITHM_OFFSET..ATT_KEY_ALGORITHM_OFFSET + 4]
            .copy_from_slice(&algorithm.to_le_bytes());
        id
    }

    #[test]
    fn read_le_u32_reads_offset_and_rejects_short_inputs() {
        let mut bytes = vec![0u8; ATT_KEY_ALGORITHM_OFFSET + 4];
        bytes[ATT_KEY_ALGORITHM_OFFSET..ATT_KEY_ALGORITHM_OFFSET + 4]
            .copy_from_slice(&0x7856_3412u32.to_le_bytes());

        assert_eq!(
            read_le_u32(&bytes, ATT_KEY_ALGORITHM_OFFSET),
            Some(0x7856_3412)
        );
        assert_eq!(
            read_le_u32(
                &bytes[..ATT_KEY_ALGORITHM_OFFSET + 3],
                ATT_KEY_ALGORITHM_OFFSET
            ),
            None
        );
    }

    #[test]
    fn select_ecdsa_p256_key_id_skips_short_and_non_ecdsa_keys() {
        let rsa_key = att_key_id(1);
        let ecdsa_key = att_key_id(SGX_QL_ALG_ECDSA_P256);

        assert_eq!(
            select_ecdsa_p256_key_id(vec![vec![0u8; 12], rsa_key, ecdsa_key.clone()]),
            Some(ecdsa_key)
        );
    }

    #[cfg(not(target_env = "sgx"))]
    #[test]
    fn attest_reports_unsupported_outside_sgx() {
        assert!(attest([0u8; CHALLENGE_LEN])
            .expect_err("attest should require SGX")
            .contains("Fortanix SGX target"));
    }

    #[cfg(target_env = "sgx")]
    #[test]
    fn remote_attestation_quote_contains_report_data_when_aesm_proxy_is_available() {
        let mut report_data = [0u8; CHALLENGE_LEN];
        for (index, byte) in report_data.iter_mut().enumerate() {
            *byte = index as u8 ^ 0xa5;
        }

        let quote = match attest(report_data) {
            Ok(quote) => quote,
            Err(err) if err.contains("connect to AESM proxy") => {
                eprintln!("skipping remote attestation smoke test: {err}");
                return;
            }
            Err(err) => panic!("remote attestation quote failed: {err}"),
        };

        assert!(!quote.is_empty());
        assert!(quote
            .windows(CHALLENGE_LEN)
            .any(|window| window == report_data.as_slice()));
    }
}
