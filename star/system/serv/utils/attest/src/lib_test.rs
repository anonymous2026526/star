#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::super::*;

    fn fake_quote(
        mrenclave: [u8; MEASUREMENT_LEN],
        mrsigner: [u8; MEASUREMENT_LEN],
        public_key: [u8; PUBLIC_KEY_LEN],
        debug: bool,
    ) -> Vec<u8> {
        let mut quote = alloc::vec![0u8; QUOTE_HEADER_LEN + SGX_REPORT_BODY_LEN];
        quote[0..2].copy_from_slice(&QUOTE_V3.to_le_bytes());
        quote[2..4].copy_from_slice(&SGX_QL_ALG_ECDSA_P256.to_le_bytes());
        let report = QUOTE_HEADER_LEN;
        let flags = if debug { 0x2u64 } else { 0 };
        quote[report + 48..report + 56].copy_from_slice(&flags.to_le_bytes());
        quote[report + 64..report + 96].copy_from_slice(&mrenclave);
        quote[report + 128..report + 160].copy_from_slice(&mrsigner);
        quote[report + 320..report + 352].copy_from_slice(&public_key);
        quote
    }

    #[test]
    fn verifies_public_key_policy() {
        let mrenclave = [1u8; MEASUREMENT_LEN];
        let mrsigner = [2u8; MEASUREMENT_LEN];
        let public_key = [3u8; PUBLIC_KEY_LEN];
        let quote = fake_quote(mrenclave, mrsigner, public_key, true);

        let policy = QuotePolicy {
            expected_mrenclave: Some(mrenclave),
            expected_mrsigner: None,
            allow_debug: true,
            allow_advisory: false,
        };

        assert_eq!(
            verify_quote_public_key(&quote, &policy).expect("verify quote"),
            public_key
        );
    }

    #[test]
    fn policy_rejects_wrong_mrenclave() {
        let quote = fake_quote(
            [1u8; MEASUREMENT_LEN],
            [2u8; MEASUREMENT_LEN],
            [3u8; PUBLIC_KEY_LEN],
            false,
        );
        let policy = QuotePolicy {
            expected_mrenclave: Some([9u8; MEASUREMENT_LEN]),
            expected_mrsigner: None,
            allow_debug: false,
            allow_advisory: false,
        };

        assert!(verify_quote_public_key(&quote, &policy).is_err());
    }

    #[test]
    fn policy_and_quote_round_trip() {
        let policy = QuotePolicy {
            expected_mrenclave: Some([8u8; MEASUREMENT_LEN]),
            expected_mrsigner: Some([7u8; MEASUREMENT_LEN]),
            allow_debug: true,
            allow_advisory: true,
        };
        let quote = [9u8; 12];

        let encoded = encode_policy_and_quote(&policy, &quote);
        let (decoded_policy, decoded_quote) =
            decode_policy_and_quote(&encoded).expect("decode policy");

        assert_eq!(decoded_policy, policy);
        assert_eq!(decoded_quote, quote);
    }

    #[test]
    fn raw_quote_evidence_round_trips() {
        let quote = [1u8; 64];
        let collateral = [2u8; 96];
        let evidence = RawQuoteEvidence {
            quote: &quote,
            collateral: &collateral,
        };

        let encoded = encode_raw_quote_evidence(&evidence);
        let decoded = decode_raw_quote_evidence(&encoded).expect("decode evidence");

        assert_eq!(decoded, evidence);
    }
}
