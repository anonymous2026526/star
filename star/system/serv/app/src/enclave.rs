use std::error::Error;
use std::fmt;
use std::io;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex};

use crate::env_or_default;
use crate::sgx::{LibraryRuntime, build_sgxs};

const STAR_ENCLAVE_SGXS: &str = "STAR_ENCLAVE_SGXS";
const STAR_ENCLAVE_HEAP_SIZE: &str = "STAR_ENCLAVE_HEAP_SIZE";
const STAR_ENCLAVE_STACK_SIZE: &str = "STAR_ENCLAVE_STACK_SIZE";
const STAR_ENCLAVE_THREADS: &str = "STAR_ENCLAVE_THREADS";

const DEFAULT_ENCLAVE_HEAP_SIZE: &str = "33554432";
const DEFAULT_ENCLAVE_STACK_SIZE: &str = "131072";

pub const ATTESTATION_REPORT_DATA_LEN: usize = 64;
pub const ENCLAVE_PUBLIC_KEY_LEN: usize = 32;

const OP_INIT: u64 = 0;
const OP_PUBLIC_KEY: u64 = 1;
const OP_ISSUE_CRED: u64 = 2;
const OP_ISSUE_TOKEN: u64 = 3;
const OP_ATTEST: u64 = 4;
const OP_TRANSFER_PUBLIC_KEY: u64 = 5;
const OP_INSTALL_KEYS: u64 = 6;
const OP_ONE: u64 = 7;

const PUBLIC_KEY_RESPONSE_CAPACITY: usize = 32;
const ISSUE_CRED_RESPONSE_CAPACITY: usize = 128;
const ISSUE_TOKEN_RESPONSE_CAPACITY: usize = 32;
const ATTEST_RESPONSE_CAPACITY: usize = 32;
const INSTALL_KEYS_RESPONSE_CAPACITY: usize = 0;
const ISSUE_TOKEN_REQUEST_CAPACITY: usize = 8 + 192;
const ONE_RESPONSE_CAPACITY: usize = ISSUE_TOKEN_RESPONSE_CAPACITY;
const ONE_REQUEST_PAYLOAD: [u8; ISSUE_TOKEN_REQUEST_CAPACITY] = [0; ISSUE_TOKEN_REQUEST_CAPACITY];

pub struct EnclaveRuntimes {
    runtime: EnclaveRuntime,
    permits: Mutex<PermitState>,
    available: Condvar,
}

struct PermitState {
    available: usize,
    total: usize,
}

pub struct EnclaveLease<'a> {
    runtimes: &'a EnclaveRuntimes,
}

impl<'a> Deref for EnclaveLease<'a> {
    type Target = EnclaveRuntime;

    fn deref(&self) -> &Self::Target {
        &self.runtimes.runtime
    }
}

impl<'a> Drop for EnclaveLease<'a> {
    fn drop(&mut self) {
        let Ok(mut guard) = self.runtimes.permits.lock() else {
            return;
        };

        if guard.available < guard.total {
            guard.available += 1;
            self.runtimes.available.notify_one();
        }
    }
}

#[derive(Debug)]
pub enum EnclaveError {
    Build(io::Error),
    BuildFailed(String),
    InvalidPublicKeyLength { len: usize },
    InvalidSealedKeys(String),
    Library(String),
    KeyManagerService(String),
    Dcap(String),
    Remote { status: u64, payload: Vec<u8> },
    ResponseTooLarge { len: usize },
    Storage(io::Error),
}

impl fmt::Display for EnclaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build(err) => write!(f, "enclave build failed to start: {err}"),
            Self::BuildFailed(status) => write!(f, "enclave build failed: {status}"),
            Self::InvalidPublicKeyLength { len } => {
                write!(f, "enclave returned public key with invalid length {len}")
            }
            Self::InvalidSealedKeys(err) => write!(f, "invalid sealed keys: {err}"),
            Self::Library(err) => write!(f, "enclave library call failed: {err}"),
            Self::KeyManagerService(err) => write!(f, "key-manager service failed: {err}"),
            Self::Dcap(err) => write!(f, "DCAP evidence failed: {err}"),
            Self::Remote { status, payload } => write!(
                f,
                "enclave returned status {status} with {} bytes",
                payload.len()
            ),
            Self::ResponseTooLarge { len } => {
                write!(f, "enclave response is too large: {len}")
            }
            Self::Storage(err) => write!(f, "sealed key storage failed: {err}"),
        }
    }
}

impl Error for EnclaveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Build(err) => Some(err),
            Self::BuildFailed(_) => None,
            Self::InvalidPublicKeyLength { .. } => None,
            Self::InvalidSealedKeys(_) => None,
            Self::Library(_) => None,
            Self::KeyManagerService(_) => None,
            Self::Dcap(_) => None,
            Self::Remote { .. } => None,
            Self::ResponseTooLarge { .. } => None,
            Self::Storage(err) => Some(err),
        }
    }
}

impl EnclaveRuntimes {
    pub fn init(num: u8) -> Result<Self, EnclaveError> {
        let runtime = EnclaveRuntime::from_env()?;
        let total = (num as usize).min(enclave_thread_count());

        Ok(EnclaveRuntimes {
            runtime,
            permits: Mutex::new(PermitState {
                available: total,
                total,
            }),
            available: Condvar::new(),
        })
    }

    pub fn from_env(num: u8) -> Result<Self, EnclaveError> {
        Self::init(num)
    }

    pub async fn pick(&self) -> Option<EnclaveLease<'_>> {
        self.pick_blocking()
    }

    pub fn pick_blocking(&self) -> Option<EnclaveLease<'_>> {
        let mut guard = self.permits.lock().ok()?;

        if guard.total == 0 {
            return None;
        }

        while guard.available == 0 {
            guard = self.available.wait(guard).ok()?;
        }

        guard.available -= 1;

        Some(EnclaveLease { runtimes: self })
    }

    pub fn available_count(&self) -> usize {
        match self.permits.lock() {
            Ok(guard) => guard.available,
            Err(_) => 0,
        }
    }

    pub fn total_count(&self) -> usize {
        match self.permits.lock() {
            Ok(guard) => guard.total,
            Err(_) => 0,
        }
    }

    pub fn transfer_key_attestation(&self) -> Result<Vec<u8>, EnclaveError> {
        self.runtime.transfer_key_attestation()
    }

    pub fn install_keys(
        &self,
        manager_evidence: &[u8],
        envelope: &[u8],
    ) -> Result<(), EnclaveError> {
        self.runtime.install_keys(manager_evidence, envelope)
    }
}

pub struct EnclaveRuntime {
    library: LibraryRuntime,
}

impl EnclaveRuntime {
    pub fn from_env() -> Result<Self, EnclaveError> {
        let sgxs = default_enclave_sgxs()?;
        Self::init_library(&sgxs)
    }

    fn init_library(sgxs: &PathBuf) -> Result<Self, EnclaveError> {
        let library = LibraryRuntime::load(sgxs)?;
        let runtime = EnclaveRuntime { library };
        runtime.initialize()?;
        Ok(runtime)
    }

    fn initialize(&self) -> Result<(), EnclaveError> {
        self.call(OP_INIT, &[])?;
        Ok(())
    }

    pub fn public_key(&self) -> Result<[u8; ENCLAVE_PUBLIC_KEY_LEN], EnclaveError> {
        let response = self.call(OP_PUBLIC_KEY, &[])?;
        let len = response.len();

        response
            .as_slice()
            .try_into()
            .map_err(|_| EnclaveError::InvalidPublicKeyLength { len })
    }

    pub fn issue_cred(&self, cipher: &[u8]) -> Result<Vec<u8>, EnclaveError> {
        self.call(OP_ISSUE_CRED, cipher)
    }

    pub fn issue_token(
        &self,
        cipher: &[u8],
        current_period: [u8; 8],
    ) -> Result<Vec<u8>, EnclaveError> {
        let mut payload = Vec::with_capacity(current_period.len() + cipher.len());
        payload.extend_from_slice(&current_period);
        payload.extend_from_slice(cipher);

        self.call(OP_ISSUE_TOKEN, &payload)
    }

    pub fn transfer_public_key(&self) -> Result<[u8; ENCLAVE_PUBLIC_KEY_LEN], EnclaveError> {
        let response = self.call(OP_TRANSFER_PUBLIC_KEY, &[])?;
        let len = response.len();

        response
            .as_slice()
            .try_into()
            .map_err(|_| EnclaveError::InvalidPublicKeyLength { len })
    }

    pub fn transfer_key_attestation(&self) -> Result<Vec<u8>, EnclaveError> {
        self.call(OP_ATTEST, &[])
    }

    pub fn install_keys(
        &self,
        manager_evidence: &[u8],
        envelope: &[u8],
    ) -> Result<(), EnclaveError> {
        let payload = encode_install_request(manager_evidence, envelope);
        self.call(OP_INSTALL_KEYS, &payload)?;
        Ok(())
    }

    pub fn one(&self) -> Result<Vec<u8>, EnclaveError> {
        self.call(OP_ONE, &ONE_REQUEST_PAYLOAD)
    }

    fn call(&self, op: u64, payload: &[u8]) -> Result<Vec<u8>, EnclaveError> {
        self.library
            .call(op, payload, initial_response_capacity(op, payload.len()))
    }
}

pub fn report_data_for_public_key(
    public_key: [u8; ENCLAVE_PUBLIC_KEY_LEN],
) -> [u8; ATTESTATION_REPORT_DATA_LEN] {
    star_attestation::report_data_for_public_key(public_key)
}

fn default_enclave_sgxs() -> Result<PathBuf, EnclaveError> {
    if let Some(path) = std::env::var_os(STAR_ENCLAVE_SGXS) {
        return Ok(PathBuf::from(path));
    }

    let manifest_dir = match option_env!("CARGO_MANIFEST_DIR") {
        Some(dir) => dir,
        None => ".",
    };

    let enclave_dir = PathBuf::from(manifest_dir).join("../enclave");
    let manifest_path = enclave_dir.join("Cargo.toml");

    build_enclave_sgxs(&manifest_path)
}

fn build_enclave_sgxs(manifest_path: &Path) -> Result<PathBuf, EnclaveError> {
    build_sgxs(
        manifest_path,
        "enclave",
        env_or_default(STAR_ENCLAVE_HEAP_SIZE, DEFAULT_ENCLAVE_HEAP_SIZE),
        env_or_default(STAR_ENCLAVE_STACK_SIZE, DEFAULT_ENCLAVE_STACK_SIZE),
        default_enclave_threads(),
    )
}

fn default_enclave_threads() -> String {
    std::env::var(STAR_ENCLAVE_THREADS)
        .unwrap_or_else(|_| default_enclave_thread_count().to_string())
}

fn enclave_thread_count() -> usize {
    std::env::var(STAR_ENCLAVE_THREADS)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or_else(default_enclave_thread_count)
}

fn default_enclave_thread_count() -> usize {
    std::thread::available_parallelism()
        .map(|threads| threads.get())
        .unwrap_or(1)
}

fn initial_response_capacity(op: u64, payload_len: usize) -> usize {
    match op {
        OP_INIT if payload_len == 0 => 0,
        OP_INIT => 0,
        OP_PUBLIC_KEY => PUBLIC_KEY_RESPONSE_CAPACITY,
        OP_ISSUE_CRED => ISSUE_CRED_RESPONSE_CAPACITY,
        OP_ISSUE_TOKEN => ISSUE_TOKEN_RESPONSE_CAPACITY,
        OP_ATTEST => ATTEST_RESPONSE_CAPACITY,
        OP_TRANSFER_PUBLIC_KEY => PUBLIC_KEY_RESPONSE_CAPACITY,
        OP_INSTALL_KEYS => INSTALL_KEYS_RESPONSE_CAPACITY,
        OP_ONE => ONE_RESPONSE_CAPACITY,
        _ => 1024,
    }
}

fn encode_install_request(manager_evidence: &[u8], envelope: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    star_attestation::append_blob(&mut out, manager_evidence);
    star_attestation::append_blob(&mut out, envelope);
    out
}
