pub mod aesm_proxy;
pub mod local_http;
pub mod retry;

use std::time::Duration;

pub fn manifest_path(relative_path: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), relative_path)
}
