use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use aesm_client::AesmClient;
use enclave_runner::{EnclaveBuilder, Library};
use enclave_runner_sgx::EnclaveBuilder as SgxEnclaveBuilder;
use sgxs_loaders::isgx::Device as SgxDevice;

use crate::enclave::EnclaveError;

pub(crate) const SGX_TARGET: &str = "x86_64-fortanix-unknown-sgx";

const MAX_PAYLOAD_LEN: usize = 16 * 1024 * 1024;
const STATUS_OK: u64 = 0;
const STATUS_BUFFER_TOO_SMALL: u64 = 5;

pub(crate) struct LibraryRuntime {
    library: Library,
}

unsafe impl Send for LibraryRuntime {}
unsafe impl Sync for LibraryRuntime {}

impl LibraryRuntime {
    pub(crate) fn load(sgxs: &Path) -> Result<Self, EnclaveError> {
        let mut device = SgxDevice::new()
            .map_err(EnclaveError::Build)?
            .einittoken_provider(AesmClient::new())
            .build();

        let sgx_builder = SgxEnclaveBuilder::new(sgxs);

        let library = EnclaveBuilder::new(sgx_builder)
            .build(&mut device)
            .map_err(|err| EnclaveError::Library(err.to_string()))?;

        Ok(Self { library })
    }

    pub(crate) fn call(
        &self,
        op: u64,
        payload: &[u8],
        initial_capacity: usize,
    ) -> Result<Vec<u8>, EnclaveError> {
        std::thread::scope(|scope| {
            let handle = scope.spawn(|| call_library(&self.library, op, payload, initial_capacity));
            match handle.join() {
                Ok(result) => result,
                Err(panic) => std::panic::resume_unwind(panic),
            }
        })
    }
}

pub(crate) fn build_sgxs(
    manifest_path: &Path,
    binary_name: &str,
    heap_size: String,
    stack_size: String,
    threads: String,
) -> Result<PathBuf, EnclaveError> {
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        EnclaveError::BuildFailed(format!(
            "missing parent directory for {}",
            manifest_path.display()
        ))
    })?;

    run_command(
        Command::new("cargo")
            .current_dir(manifest_dir)
            .arg("build")
            .arg("--quiet")
            .arg("--release")
            .arg("--target")
            .arg(SGX_TARGET)
            .arg("--manifest-path")
            .arg(manifest_path),
    )?;

    let target_dir = release_target_dir(manifest_dir, binary_name)?;
    let elf_path = target_dir.join(binary_name);
    let sgxs_path = target_dir.join(format!("{binary_name}.sgxs"));

    run_command(
        Command::new("ftxsgx-elf2sgxs")
            .arg(&elf_path)
            .arg("--library")
            //.arg("--debug")
            .arg("--heap-size")
            .arg(heap_size)
            .arg("--stack-size")
            .arg(stack_size)
            .arg("--threads")
            .arg(threads)
            .arg("--output")
            .arg(&sgxs_path),
    )?;

    Ok(sgxs_path)
}

fn release_target_dir(manifest_dir: &Path, binary_name: &str) -> Result<PathBuf, EnclaveError> {
    let mut candidates = Vec::new();

    if let Some(target_dir) = std::env::var_os("CARGO_TARGET_DIR") {
        candidates.push(PathBuf::from(target_dir));
    }

    let mut current = Some(manifest_dir);
    while let Some(path) = current {
        candidates.push(path.join("target"));
        current = path.parent();
    }

    for candidate in candidates {
        let target_dir = candidate.join(SGX_TARGET).join("release");
        if target_dir.join(binary_name).is_file() {
            return Ok(target_dir);
        }
    }

    Err(EnclaveError::BuildFailed(format!(
        "cargo built {binary_name}, but the SGX ELF was not found"
    )))
}

fn run_command(command: &mut Command) -> Result<(), EnclaveError> {
    let output = command.output().map_err(EnclaveError::Build)?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    Err(EnclaveError::BuildFailed(format!(
        "{}\nstdout:\n{}\nstderr:\n{}",
        output.status, stdout, stderr
    )))
}

fn call_library(
    library: &Library,
    op: u64,
    payload: &[u8],
    initial_capacity: usize,
) -> Result<Vec<u8>, EnclaveError> {
    let mut output = vec![0u8; initial_capacity];

    loop {
        let (status, len) = unsafe {
            library.call(
                op,
                payload.as_ptr() as u64,
                payload.len() as u64,
                output.as_mut_ptr() as u64,
                output.len() as u64,
            )
        }
        .map_err(|err| EnclaveError::Library(err.to_string()))?;

        let len =
            usize::try_from(len).map_err(|_| EnclaveError::ResponseTooLarge { len: usize::MAX })?;

        if status == STATUS_BUFFER_TOO_SMALL {
            if len > MAX_PAYLOAD_LEN {
                return Err(EnclaveError::ResponseTooLarge { len });
            }

            output.resize(len, 0);
            continue;
        }

        if len > output.len() {
            return Err(EnclaveError::ResponseTooLarge { len });
        }

        output.truncate(len);

        if status == STATUS_OK {
            return Ok(output);
        }

        return Err(EnclaveError::Remote {
            status,
            payload: output,
        });
    }
}
