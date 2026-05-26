use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use key_manager_client::ENCLAVE_PUBLIC_KEY_LEN;

use crate::sgx::{build_sgxs, EnclaveError, LibraryRuntime};

const STAR_KEY_MANAGER_SGXS: &str = "STAR_KEY_MANAGER_SGXS";
const STAR_KEY_MANAGER_HEAP_SIZE: &str = "STAR_KEY_MANAGER_HEAP_SIZE";
const STAR_KEY_MANAGER_STACK_SIZE: &str = "STAR_KEY_MANAGER_STACK_SIZE";
const STAR_KEY_MANAGER_THREADS: &str = "STAR_KEY_MANAGER_THREADS";
const STAR_KEY_MANAGER_SEALED_KEYS: &str = "STAR_KEY_MANAGER_SEALED_KEYS";
const DEFAULT_KEY_MANAGER_SEALED_KEYS: &str = "key-manager.keys.sealed";

const DEFAULT_KEY_MANAGER_HEAP_SIZE: &str = "33554432";
const DEFAULT_KEY_MANAGER_STACK_SIZE: &str = "131072";

const OP_INIT: u64 = 0;
const OP_PUBLIC_KEY: u64 = 1;
const OP_ATTEST: u64 = 4;
const OP_EXPORT_KEYS: u64 = 5;

const PUBLIC_KEY_RESPONSE_CAPACITY: usize = 32;
const ATTEST_RESPONSE_CAPACITY: usize = 32;
const EXPORT_KEYS_RESPONSE_CAPACITY: usize = 256;
const INIT_RESPONSE_CAPACITY: usize = 1024;

pub struct KeyManagerRuntime {
    library: LibraryRuntime,
    sealed_keys: Vec<u8>,
}

impl KeyManagerRuntime {
    pub fn from_env() -> Result<Self, EnclaveError> {
        let sgxs = default_key_manager_sgxs()?;
        let library = LibraryRuntime::load(&sgxs)?;
        let sealed_keys_path = sealed_keys_path();
        let init_payload = read_optional_sealed_keys(&sealed_keys_path)?;
        let sealed_keys = library.call(
            OP_INIT,
            init_payload.as_deref().unwrap_or(&[]),
            initial_response_capacity(OP_INIT),
        )?;

        if init_payload.as_deref() != Some(sealed_keys.as_slice()) {
            write_sealed_keys(&sealed_keys_path, &sealed_keys)?;
        }

        Ok(KeyManagerRuntime {
            library,
            sealed_keys,
        })
    }

    pub fn from_env_with_sealed_keys(sealed_keys: &[u8]) -> Result<Self, EnclaveError> {
        let sgxs = default_key_manager_sgxs()?;
        let library = LibraryRuntime::load(&sgxs)?;
        let sealed_keys = library.call(OP_INIT, sealed_keys, initial_response_capacity(OP_INIT))?;

        Ok(KeyManagerRuntime {
            library,
            sealed_keys,
        })
    }

    pub fn sealed_keys(&self) -> &[u8] {
        &self.sealed_keys
    }

    pub fn public_key(&self) -> Result<[u8; ENCLAVE_PUBLIC_KEY_LEN], EnclaveError> {
        let response = self.call(OP_PUBLIC_KEY, &[])?;
        let len = response.len();

        response
            .as_slice()
            .try_into()
            .map_err(|_| EnclaveError::Library(format!("invalid public key length: {len}")))
    }

    pub fn public_key_attestation(&self) -> Result<Vec<u8>, EnclaveError> {
        self.call(OP_ATTEST, &[])
    }

    pub fn export_keys(&self, recipient_evidence: &[u8]) -> Result<Vec<u8>, EnclaveError> {
        self.call(OP_EXPORT_KEYS, recipient_evidence)
    }

    fn call(&self, op: u64, payload: &[u8]) -> Result<Vec<u8>, EnclaveError> {
        self.library
            .call(op, payload, initial_response_capacity(op))
    }
}

fn initial_response_capacity(op: u64) -> usize {
    match op {
        OP_INIT => INIT_RESPONSE_CAPACITY,
        OP_PUBLIC_KEY => PUBLIC_KEY_RESPONSE_CAPACITY,
        OP_ATTEST => ATTEST_RESPONSE_CAPACITY,
        OP_EXPORT_KEYS => EXPORT_KEYS_RESPONSE_CAPACITY,
        _ => 1024,
    }
}

fn sealed_keys_path() -> PathBuf {
    std::env::var_os(STAR_KEY_MANAGER_SEALED_KEYS)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_KEY_MANAGER_SEALED_KEYS))
}

fn read_optional_sealed_keys(path: &Path) -> Result<Option<Vec<u8>>, EnclaveError> {
    match fs::read(path) {
        Ok(sealed_keys) => Ok(Some(sealed_keys)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(EnclaveError::Library(format!("read sealed keys: {err}"))),
    }
}

fn write_sealed_keys(path: &Path, sealed_keys: &[u8]) -> Result<(), EnclaveError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|err| EnclaveError::Library(format!("create sealed keys dir: {err}")))?;
    }

    fs::write(path, sealed_keys).map_err(|err| EnclaveError::Library(format!("write sealed keys: {err}")))
}

fn default_key_manager_sgxs() -> Result<PathBuf, EnclaveError> {
    if let Some(path) = std::env::var_os(STAR_KEY_MANAGER_SGXS) {
        return Ok(PathBuf::from(path));
    }

    let manifest_dir = match option_env!("CARGO_MANIFEST_DIR") {
        Some(dir) => dir,
        None => ".",
    };

    let key_manager_dir = PathBuf::from(manifest_dir).join("../../manage-key/enclave");
    let manifest_path = key_manager_dir.join("Cargo.toml");

    build_key_manager_sgxs(&manifest_path)
}

fn env_or_default(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

fn build_key_manager_sgxs(manifest_path: &Path) -> Result<PathBuf, EnclaveError> {
    build_sgxs(
        manifest_path,
        "key-manager",
        env_or_default(STAR_KEY_MANAGER_HEAP_SIZE, DEFAULT_KEY_MANAGER_HEAP_SIZE),
        env_or_default(STAR_KEY_MANAGER_STACK_SIZE, DEFAULT_KEY_MANAGER_STACK_SIZE),
        default_key_manager_threads(),
    )
}

fn default_key_manager_threads() -> String {
    std::env::var(STAR_KEY_MANAGER_THREADS).unwrap_or_else(|_| "1".to_string())
}
