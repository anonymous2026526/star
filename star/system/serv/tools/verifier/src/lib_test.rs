#[cfg(test)]
mod tests {
    use super::super::*;

    const QUOTE_HEADER_LEN: usize = 48;
    const SGX_REPORT_BODY_LEN: usize = 384;
    const QUOTE_V3: u16 = 3;
    const AUTH_DATA_V3_LEN: usize = 64 + 64 + 384 + 64 + 2 + 2 + 4;

    fn fake_quote(
        mrenclave: [u8; MEASUREMENT_LEN],
        mrsigner: [u8; MEASUREMENT_LEN],
        public_key: [u8; PUBLIC_KEY_LEN],
        debug: bool,
    ) -> Vec<u8> {
        let auth_data_len_offset = QUOTE_HEADER_LEN + SGX_REPORT_BODY_LEN;
        let mut quote = vec![0u8; auth_data_len_offset + 4 + AUTH_DATA_V3_LEN];
        quote[0..2].copy_from_slice(&QUOTE_V3.to_le_bytes());
        quote[2..4].copy_from_slice(&SGX_QL_ALG_ECDSA_P256.to_le_bytes());

        let report = QUOTE_HEADER_LEN;
        let flags = if debug { 0x2u64 } else { 0 };
        quote[report + 48..report + 56].copy_from_slice(&flags.to_le_bytes());
        quote[report + 64..report + 96].copy_from_slice(&mrenclave);
        quote[report + 128..report + 160].copy_from_slice(&mrsigner);
        quote[report + 320..report + 352].copy_from_slice(&public_key);
        quote[auth_data_len_offset..auth_data_len_offset + 4]
            .copy_from_slice(&(AUTH_DATA_V3_LEN as u32).to_le_bytes());
        quote
    }

    #[test]
    fn parses_quote_report_fields() {
        let mrenclave = [1u8; MEASUREMENT_LEN];
        let mrsigner = [2u8; MEASUREMENT_LEN];
        let public_key = [3u8; PUBLIC_KEY_LEN];

        let parsed =
            parse_quote(&fake_quote(mrenclave, mrsigner, public_key, true)).expect("parse quote");

        assert_eq!(parsed.mrenclave, mrenclave);
        assert_eq!(parsed.mrsigner, mrsigner);
        assert_eq!(
            public_key_from_report_data(&parsed).expect("public key"),
            public_key
        );
        assert!(parsed.debug);
    }

    #[test]
    fn policy_rejects_untrusted_debug_quote() {
        let mrenclave = [1u8; MEASUREMENT_LEN];
        let mrsigner = [2u8; MEASUREMENT_LEN];
        let public_key = [3u8; PUBLIC_KEY_LEN];
        let parsed =
            parse_quote(&fake_quote(mrenclave, mrsigner, public_key, true)).expect("parse quote");

        let err = verify_policy(&parsed, &mrsigner, Some(&mrenclave), false)
            .expect_err("debug quote should be rejected");

        assert!(err.contains("debug enclave"));
    }

    #[test]
    fn hex_round_trips_and_accepts_separators() {
        let parsed = parse_hex_array::<4>("0x12:34_56 78", "test").expect("parse hex");
        assert_eq!(parsed, [0x12, 0x34, 0x56, 0x78]);
        assert_eq!(hex(&parsed), "12345678");
    }

    #[test]
    fn names_dcap_statuses() {
        assert_eq!(dcap_status_name(TcbStatus::UpToDate), "UpToDate");
        assert_eq!(dcap_status_name(TcbStatus::OutOfDate), "OutOfDate");
        assert!(dcap_status_is_advisory(TcbStatus::OutOfDate));
        assert!(!dcap_status_is_advisory(TcbStatus::Revoked));
    }

    #[test]
    fn extracts_base64_quote_from_startup_log_line() {
        let quote = fake_quote(
            [1u8; MEASUREMENT_LEN],
            [2u8; MEASUREMENT_LEN],
            [3u8; PUBLIC_KEY_LEN],
            false,
        );
        let input = format!(
            "STAR_ATTESTATION_QUOTE_BASE64={}\n",
            URL_SAFE.encode(&quote)
        );

        let decoded = decode_quote_input(input.as_bytes()).expect("decode quote input");

        assert_eq!(decoded, quote);
    }

    #[test]
    fn extracts_padded_base64_quote_from_startup_log_line() {
        let mut quote = fake_quote(
            [1u8; MEASUREMENT_LEN],
            [2u8; MEASUREMENT_LEN],
            [3u8; PUBLIC_KEY_LEN],
            false,
        );
        quote.push(9);
        let input = format!(
            "STAR_ATTESTATION_QUOTE_BASE64={}\n",
            URL_SAFE.encode(&quote)
        );

        let decoded = decode_quote_input(input.as_bytes()).expect("decode quote input");

        assert_eq!(decoded, quote);
    }
}
