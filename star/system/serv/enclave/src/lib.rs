#![cfg_attr(not(target_env = "sgx"), allow(dead_code))]

mod attest;

use hpke::{
    aead::ChaCha20Poly1305 as HpkeChaCha20Poly1305, kdf::HkdfSha256, kem::X25519HkdfSha256,
    setup_receiver, Deserializable, HpkeError, OpModeR,
};
use rand::rngs::OsRng;
#[cfg(target_env = "sgx")]
use sgx_isa::{AttributesFlags, Report};
#[cfg(target_env = "sgx")]
use star_attestation::parse_measurement_hex;
use star_attestation::{
    report_data_for_public_key, take_blob, verify_raw_quote_evidence_public_key, QuotePolicy,
    PUBLIC_KEY_LEN,
};
use star_core::{secure_channel::server::SecureChannelServer, serv::enclave::Enclave};
#[cfg(target_env = "sgx")]
use std::slice;
#[cfg(target_env = "sgx")]
use std::sync::{OnceLock, RwLock};

const PERIOD_LEN: usize = 8;
// This policy value must be embedded in the enclave image so that changing the
// per-credential request limit changes MRENCLAVE. Normal builds use the
// generated config; benchmark builds use an explicit cargo feature.
#[cfg(not(feature = "bench-max-count"))]
include!(concat!(env!("OUT_DIR"), "/max_count.rs"));
#[cfg(feature = "bench-max-count")]
const MAX_COUNT: u32 = 1_000_000;
#[used]
static MEASURED_MAX_COUNT_BE: [u8; 4] = MAX_COUNT.to_be_bytes();
const MAX_PAYLOAD_LEN: usize = 16 * 1024 * 1024;

const OP_INIT: u8 = 0;
const OP_PUBLIC_KEY: u8 = 1;
const OP_ISSUE_CRED: u8 = 2;
const OP_ISSUE_TOKEN: u8 = 3;
const OP_ATTEST: u8 = 4;
const OP_TRANSFER_PUBLIC_KEY: u8 = 5;
const OP_INSTALL_KEYS: u8 = 6;
const OP_ONE: u8 = 7;

const STATUS_OK: u8 = 0;
const STATUS_NOT_INITIALIZED: u8 = 1;
const STATUS_BAD_REQUEST: u8 = 2;
const STATUS_PANIC: u8 = 4;
const STATUS_BUFFER_TOO_SMALL: u8 = 5;

// EnclaveApp manages the enclave state and dispatches operations.
struct EnclaveApp {
    enclave: Option<Enclave>,
    transfer_receiver: TransferReceiver,
}

impl EnclaveApp {
    fn new() -> Self {
        Self {
            enclave: None,
            transfer_receiver: TransferReceiver::random(&mut OsRng),
        }
    }

    #[cfg(test)]
    fn with_enclave(enclave: Enclave) -> Self {
        Self {
            enclave: Some(enclave),
            transfer_receiver: TransferReceiver::random(&mut OsRng),
        }
    }

    /// OP_INIT never installs production keys.
    ///
    /// Each startup creates a fresh attested transfer key. The host must fetch
    /// that key, ask key-manager to export the production keys for this enclave
    /// instance, and deliver the key-manager envelope through OP_INSTALL_KEYS.
    /// This avoids accepting a raw key bundle from the host.
    fn init(&mut self, payload: &[u8]) -> (u8, Vec<u8>) {
        if self.enclave.is_some() {
            return (STATUS_BAD_REQUEST, b"enclave already initialized".to_vec());
        }

        if !payload.is_empty() {
            return (
                STATUS_BAD_REQUEST,
                b"init does not accept keys; install key-manager envelope with OP_INSTALL_KEYS"
                    .to_vec(),
            );
        }

        self.transfer_receiver = TransferReceiver::random(&mut OsRng);
        (STATUS_OK, Vec::new())
    }

    fn transfer_public_key(&self, _: &[u8]) -> (u8, Vec<u8>) {
        (STATUS_OK, self.transfer_receiver.public_key().to_vec())
    }

    // install_keys verifies the manager's evidence, decrypts the key-manager's envelope,
    // and initializes the enclave with the contained keys.
    // It returns a panic status if the keys are
    fn install_keys(&mut self, payload: &[u8]) -> (u8, Vec<u8>) {
        if self.enclave.is_some() {
            return (STATUS_BAD_REQUEST, b"enclave already initialized".to_vec());
        }

        let (manager_evidence, envelope) = match decode_install_request(payload) {
            Ok(request) => request,
            Err(err) => return (STATUS_BAD_REQUEST, err.into_bytes()),
        };

        let manager_policy = match trusted_key_manager_policy() {
            Ok(policy) => policy,
            Err(err) => return (STATUS_BAD_REQUEST, err.into_bytes()),
        };

        let manager_public_key =
            match verify_raw_quote_evidence_public_key(manager_evidence, &manager_policy) {
                Ok(public_key) => public_key,
                Err(err) => return (STATUS_BAD_REQUEST, err.into_bytes()),
            };

        let plaintext = match self
            .transfer_receiver
            .open_auth(envelope, manager_public_key)
        {
            Ok(plaintext) => plaintext,
            Err(err) => {
                return (
                    STATUS_BAD_REQUEST,
                    format!("decrypt key-manager envelope: {err:?}").into_bytes(),
                );
            }
        };

        let KeyBundle {
            enc_key,
            hpke_receiver_key,
        } = match decode_key_bundle(&plaintext) {
            Ok(bundle) => bundle,
            Err(err) => return (STATUS_BAD_REQUEST, err.into_bytes()),
        };

        match Enclave::new(enclave_max_count(), enc_key, hpke_receiver_key) {
            Ok(enclave) => {
                self.enclave = Some(enclave);
                (STATUS_OK, Vec::new())
            }
            Err(err) => (STATUS_PANIC, format!("{err:?}").into_bytes()),
        }
    }

    // enclave returns a reference to the initialized Enclave or an error status if not initialized.
    fn enclave(&self) -> Result<&Enclave, (u8, Vec<u8>)> {
        self.enclave.as_ref().ok_or_else(|| {
            (
                STATUS_NOT_INITIALIZED,
                b"enclave is not initialized".to_vec(),
            )
        })
    }

    fn public_key(&self, _: &[u8]) -> (u8, Vec<u8>) {
        match self.enclave() {
            Ok(enclave) => (STATUS_OK, enclave.public_key().to_vec()),
            Err(err) => err,
        }
    }

    fn issue_cred(&self, payload: &[u8]) -> (u8, Vec<u8>) {
        let Some(enclave) = self.enclave.as_ref() else {
            return (
                STATUS_NOT_INITIALIZED,
                b"enclave is not initialized".to_vec(),
            );
        };

        let mut rng = OsRng;
        let cred = enclave.issue_cred(payload, &mut rng);
        if cred.is_empty() {
            return (STATUS_BAD_REQUEST, b"issue_cred failed".to_vec());
        }
        (STATUS_OK, cred)
    }

    fn issue_token(&self, payload: &[u8]) -> (u8, Vec<u8>) {
        if payload.len() < PERIOD_LEN {
            return (STATUS_BAD_REQUEST, b"issue_token missing period".to_vec());
        }

        let current_period = payload[..PERIOD_LEN].try_into().expect("fixed slice");
        let cipher = &payload[PERIOD_LEN..];

        let enclave = match self.enclave() {
            Ok(enclave) => enclave,
            Err(err) => return err,
        };

        match enclave.issue_token(cipher, current_period) {
            Ok(token) => (STATUS_OK, token),
            Err(_) => (STATUS_BAD_REQUEST, b"issue_token failed".to_vec()),
        }
    }

    fn attest(&self, payload: &[u8]) -> (u8, Vec<u8>) {
        if !payload.is_empty() {
            return (
                STATUS_BAD_REQUEST,
                b"transfer attestation does not accept host report_data".to_vec(),
            );
        }

        let report_data = report_data_for_public_key(self.transfer_receiver.public_key());

        match attest::attest(report_data) {
            Ok(quote) => (STATUS_OK, quote),
            Err(err) => (STATUS_PANIC, err.into_bytes()),
        }
    }

    fn one(&self, _: &[u8]) -> (u8, Vec<u8>) {
        (STATUS_OK, vec![1; 32])
    }

    fn handle_shared(&self, op: u8, payload: &[u8]) -> (u8, Vec<u8>) {
        match op {
            OP_PUBLIC_KEY => self.public_key(payload),
            OP_ISSUE_CRED => self.issue_cred(payload),
            OP_ISSUE_TOKEN => self.issue_token(payload),
            OP_ATTEST => self.attest(payload),
            OP_TRANSFER_PUBLIC_KEY => self.transfer_public_key(payload),
            OP_ONE => self.one(payload),
            _ => (STATUS_BAD_REQUEST, b"unknown operation".to_vec()),
        }
    }

    fn handle(&mut self, op: u8, payload: &[u8]) -> (u8, Vec<u8>) {
        match op {
            OP_INIT => self.init(payload),
            OP_INSTALL_KEYS => self.install_keys(payload),
            _ => self.handle_shared(op, payload),
        }
    }
}

fn enclave_max_count() -> u32 {
    u32::from_be_bytes(MEASURED_MAX_COUNT_BE)
}

pub fn run() {}

#[cfg(target_env = "sgx")]
static LIBRARY_APP: OnceLock<RwLock<EnclaveApp>> = OnceLock::new();

#[cfg(target_env = "sgx")]
#[allow(improper_ctypes_definitions)]
#[no_mangle]
// Matches the Fortanix SGX library entry ABI: p1 selects the operation, p2-p5 carry buffers.
pub extern "C" fn entry(p1: u64, p2: u64, p3: u64, _ignore: u64, p4: u64, p5: u64) -> (u64, u64) {
    library_entry(p1, p2, p3, p4, p5)
}

#[cfg(target_env = "sgx")]
fn library_entry(
    op: u64,
    input_ptr: u64,
    input_len: u64,
    output_ptr: u64,
    output_len: u64,
) -> (u64, u64) {
    let op = match u8::try_from(op) {
        Ok(op) => op,
        Err(_) => return (STATUS_BAD_REQUEST as u64, 0),
    };

    let input = match unsafe { user_slice(input_ptr, input_len) } {
        Ok(input) => input,
        Err(status) => return (status as u64, 0),
    };
    let output = match unsafe { user_slice_mut(output_ptr, output_len) } {
        Ok(output) => output,
        Err(status) => return (status as u64, 0),
    };

    let app = LIBRARY_APP.get_or_init(|| RwLock::new(EnclaveApp::new()));
    let (status, response) = if matches!(op, OP_INIT | OP_INSTALL_KEYS) {
        match app.write() {
            Ok(mut app) => app.handle(op, input),
            Err(_) => (STATUS_PANIC, b"enclave app lock poisoned".to_vec()),
        }
    } else {
        match app.read() {
            Ok(app) => app.handle_shared(op, input),
            Err(_) => (STATUS_PANIC, b"enclave app lock poisoned".to_vec()),
        }
    };

    if response.len() > output.len() {
        return (STATUS_BUFFER_TOO_SMALL as u64, response.len() as u64);
    }

    output[..response.len()].copy_from_slice(&response);
    (status as u64, response.len() as u64)
}

#[cfg(target_env = "sgx")]
unsafe fn user_slice<'a>(ptr: u64, len: u64) -> Result<&'a [u8], u8> {
    let len = checked_user_len(len)?;
    if len == 0 {
        return Ok(&[]);
    }

    let ptr = ptr as *const u8;
    if ptr.is_null() || !is_user_range(ptr, len) {
        return Err(STATUS_BAD_REQUEST);
    }

    Ok(unsafe { slice::from_raw_parts(ptr, len) })
}

#[cfg(target_env = "sgx")]
unsafe fn user_slice_mut<'a>(ptr: u64, len: u64) -> Result<&'a mut [u8], u8> {
    let len = checked_user_len(len)?;
    if len == 0 {
        return Ok(&mut []);
    }

    let ptr = ptr as *mut u8;
    if ptr.is_null() || !is_user_range(ptr, len) {
        return Err(STATUS_BAD_REQUEST);
    }

    Ok(unsafe { slice::from_raw_parts_mut(ptr, len) })
}

#[cfg(target_env = "sgx")]
fn checked_user_len(len: u64) -> Result<usize, u8> {
    let len = usize::try_from(len).map_err(|_| STATUS_BAD_REQUEST)?;
    if len > MAX_PAYLOAD_LEN {
        return Err(STATUS_BAD_REQUEST);
    }
    Ok(len)
}

#[cfg(target_env = "sgx")]
fn is_user_range(ptr: *const u8, len: usize) -> bool {
    if len == 0 {
        return true;
    }

    let start = ptr as usize;
    let Some(end) = start.checked_add(len - 1) else {
        return false;
    };

    let base = image_base();
    let enclave_size = unsafe { ENCLAVE_SIZE };
    let Some(enclave_end) = base.checked_add(enclave_size.saturating_sub(1)) else {
        return false;
    };

    end < base || start > enclave_end
}

#[cfg(target_env = "sgx")]
fn image_base() -> usize {
    let base: usize;
    unsafe {
        std::arch::asm!(
            "lea IMAGE_BASE(%rip), {}",
            lateout(reg) base,
            options(att_syntax, nostack, preserves_flags, nomem, pure),
        );
    }
    base
}

#[cfg(target_env = "sgx")]
unsafe extern "C" {
    static ENCLAVE_SIZE: usize;
}

struct TransferReceiver {
    channel: SecureChannelServer,
}

impl TransferReceiver {
    fn random(rng: &mut OsRng) -> Self {
        Self {
            channel: SecureChannelServer::random(rng),
        }
    }

    fn public_key(&self) -> [u8; PUBLIC_KEY_LEN] {
        self.channel.public_key()
    }

    fn open_auth(
        &self,
        envelope: &[u8],
        sender_public_key: [u8; PUBLIC_KEY_LEN],
    ) -> Result<Vec<u8>, HpkeError> {
        const AAD_LEN: usize = 32;
        const INFO_LEN: usize = 32;
        const ENC_LEN: usize = 32;

        if envelope.len() < AAD_LEN + INFO_LEN + ENC_LEN {
            return Err(HpkeError::MessageLimitReached);
        }

        let aad = &envelope[..AAD_LEN];
        let info = &envelope[AAD_LEN..AAD_LEN + INFO_LEN];
        let enc = &envelope[AAD_LEN + INFO_LEN..AAD_LEN + INFO_LEN + ENC_LEN];
        let ciphertext = &envelope[AAD_LEN + INFO_LEN + ENC_LEN..];

        let enc = <X25519HkdfSha256 as hpke::kem::Kem>::EncappedKey::from_bytes(enc)?;
        let sender_pk =
            <X25519HkdfSha256 as hpke::kem::Kem>::PublicKey::from_bytes(&sender_public_key)?;
        let mut ctx = setup_receiver::<HpkeChaCha20Poly1305, HkdfSha256, X25519HkdfSha256>(
            &OpModeR::Auth(sender_pk),
            &self.channel.sk,
            &enc,
            info,
        )?;

        ctx.open(ciphertext, aad)
    }
}

#[derive(Clone, Copy)]
struct KeyBundle {
    enc_key: [u8; 32],
    hpke_receiver_key: [u8; 32],
}

fn decode_key_bundle(payload: &[u8]) -> Result<KeyBundle, String> {
    if payload.len() != 64 {
        return Err("key bundle must be 64 bytes".into());
    }

    Ok(KeyBundle {
        enc_key: payload[..32]
            .try_into()
            .map_err(|_| "bad enc key length".to_string())?,
        hpke_receiver_key: payload[32..64]
            .try_into()
            .map_err(|_| "bad hpke key length".to_string())?,
    })
}

fn decode_install_request(payload: &[u8]) -> Result<(&[u8], &[u8]), String> {
    let mut remaining = payload;
    let evidence = take_blob(&mut remaining)?;
    let envelope = take_blob(&mut remaining)?;
    if !remaining.is_empty() {
        return Err("trailing install request bytes".into());
    }

    Ok((evidence, envelope))
}

#[cfg(target_env = "sgx")]
fn trusted_key_manager_policy() -> Result<QuotePolicy, String> {
    let self_report = Report::for_self();
    let (expected_mrenclave, expected_mrsigner) = trusted_peer_identity(
        option_env!("STAR_TRUSTED_KEY_MANAGER_MRENCLAVE"),
        "STAR_TRUSTED_KEY_MANAGER_MRENCLAVE",
        option_env!("STAR_TRUSTED_KEY_MANAGER_MRSIGNER"),
        "STAR_TRUSTED_KEY_MANAGER_MRSIGNER",
        option_env!("STAR_TRUST_SAME_SIGNER_KEY_MANAGER"),
        self_report.mrsigner,
    )?;
    Ok(QuotePolicy {
        expected_mrenclave,
        expected_mrsigner,
        allow_debug: embedded_allow_debug(
            option_env!("STAR_ALLOW_DEBUG_KEY_MANAGER"),
            self_report
                .attributes
                .flags
                .contains(AttributesFlags::DEBUG),
        )?,
        allow_advisory: embedded_allow_advisory(option_env!("STAR_ALLOW_ADVISORY_KEY_MANAGER"))?,
    })
}

#[cfg(not(target_env = "sgx"))]
fn trusted_key_manager_policy() -> Result<QuotePolicy, String> {
    Err("key-manager policy is only available inside SGX".into())
}

#[cfg(target_env = "sgx")]
fn trusted_peer_identity(
    mrenclave: Option<&str>,
    mrenclave_name: &str,
    mrsigner: Option<&str>,
    mrsigner_name: &str,
    same_signer: Option<&str>,
    self_mrsigner: [u8; 32],
) -> Result<(Option<[u8; 32]>, Option<[u8; 32]>), String> {
    let expected_mrenclave = optional_embedded_measurement(mrenclave, mrenclave_name)?;
    let mut expected_mrsigner = optional_embedded_measurement(mrsigner, mrsigner_name)?;
    if expected_mrenclave.is_none()
        && expected_mrsigner.is_none()
        && embedded_allow_same_signer(same_signer)?
    {
        expected_mrsigner = Some(self_mrsigner);
    }
    if expected_mrenclave.is_none() && expected_mrsigner.is_none() {
        return Err(format!(
            "{mrenclave_name}, {mrsigner_name}, or STAR_TRUST_SAME_SIGNER_ENCLAVES must be embedded for peer trust"
        ));
    }
    Ok((expected_mrenclave, expected_mrsigner))
}

#[cfg(target_env = "sgx")]
fn optional_embedded_measurement(
    value: Option<&str>,
    name: &str,
) -> Result<Option<[u8; 32]>, String> {
    value
        .map(|value| parse_measurement_hex(value, name))
        .transpose()
}

#[cfg(target_env = "sgx")]
fn embedded_allow_debug(specific: Option<&str>, default: bool) -> Result<bool, String> {
    let value = specific.or(option_env!("STAR_ALLOW_DEBUG_ENCLAVES"));
    match value {
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES") => Ok(true),
        Some("0") | Some("false") | Some("FALSE") | Some("no") | Some("NO") => Ok(false),
        Some(_) => Err("invalid embedded debug policy".into()),
        None => Ok(default),
    }
}

#[cfg(target_env = "sgx")]
fn embedded_allow_advisory(specific: Option<&str>) -> Result<bool, String> {
    let value = specific.or(option_env!("STAR_ALLOW_ADVISORY_ENCLAVES"));
    match value {
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES") => Ok(true),
        Some("0") | Some("false") | Some("FALSE") | Some("no") | Some("NO") => Ok(false),
        Some(_) => Err("invalid embedded advisory policy".into()),
        None => Ok(false),
    }
}

#[cfg(target_env = "sgx")]
fn embedded_allow_same_signer(specific: Option<&str>) -> Result<bool, String> {
    let value = specific.or(option_env!("STAR_TRUST_SAME_SIGNER_ENCLAVES"));
    match value {
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES") => Ok(true),
        Some("0") | Some("false") | Some("FALSE") | Some("no") | Some("NO") => Ok(false),
        Some(_) => Err("invalid embedded same-signer policy".into()),
        None => Ok(false),
    }
}

#[path = "lib_test.rs"]
#[cfg(test)]
mod tests;
