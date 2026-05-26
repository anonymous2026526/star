#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn key_bundle_round_trips() {
        let bundle = KeyBundle {
            enc_key: [1u8; 32],
            hpke_receiver_key: [2u8; 32],
        };

        let encoded = encode_key_bundle(bundle);
        let decoded = decode_key_bundle(&encoded).expect("decode");

        assert_eq!(decoded.enc_key, bundle.enc_key);
        assert_eq!(decoded.hpke_receiver_key, bundle.hpke_receiver_key);
    }

    #[cfg(not(target_env = "sgx"))]
    #[test]
    fn init_rejects_plain_key_bundle_outside_sgx() {
        let mut app = KeyManagerApp::new();
        let plaintext = encode_key_bundle(KeyBundle {
            enc_key: [1u8; 32],
            hpke_receiver_key: [2u8; 32],
        });

        let (status, payload) = app.init(&plaintext);

        assert_eq!(status, STATUS_BAD_REQUEST);
        assert!(String::from_utf8(payload)
            .unwrap()
            .contains("unseal key bundle"));
    }

    #[cfg(not(target_env = "sgx"))]
    #[test]
    fn public_key_attest_reports_unsupported_outside_sgx() {
        let app = KeyManagerApp::new();
        let (status, payload) = app.attest(&[]);

        assert_eq!(status, STATUS_PANIC);
        assert!(String::from_utf8(payload).unwrap().contains("Fortanix SGX"));
    }
}
