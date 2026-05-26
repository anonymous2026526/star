use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use star_attestation::{append_blob, take_blob};

pub const DEFAULT_KEY_MANAGER_PUBLIC_KEY_API: &str = "/star/key-manager/public_key";
pub const DEFAULT_KEY_MANAGER_ATTESTATION_API: &str = "/star/key-manager/attestation";
pub const DEFAULT_KEY_MANAGER_EXPORT_API: &str = "/star/key-manager/export";
pub const STAR_KEY_MANAGER_URL: &str = "STAR_KEY_MANAGER_URL";
pub const ENCLAVE_PUBLIC_KEY_LEN: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum KeyManagerError {
    #[error("enclave error: {0}")]
    Enclave(String),
    #[error("key-manager returned public key with invalid length {len}")]
    InvalidPublicKeyLength { len: usize },
    #[error("service error: {0}")]
    Service(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyManagerExport {
    pub key_manager_quote: Vec<u8>,
    pub recipient_evidence: Vec<u8>,
    pub envelope: Vec<u8>,
}

pub struct RemoteKeyManagerRuntime {
    base_url: String,
    agent: ureq::Agent,
}

impl RemoteKeyManagerRuntime {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            agent: ureq::AgentBuilder::new().build(),
        }
    }

    pub fn from_env() -> Option<Self> {
        std::env::var(STAR_KEY_MANAGER_URL)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .map(Self::new)
    }

    pub fn public_key(&self) -> Result<[u8; ENCLAVE_PUBLIC_KEY_LEN], KeyManagerError> {
        let response = self.get_bytes(DEFAULT_KEY_MANAGER_PUBLIC_KEY_API)?;
        let len = response.len();

        response
            .as_slice()
            .try_into()
            .map_err(|_| KeyManagerError::InvalidPublicKeyLength { len })
    }

    pub fn public_key_attestation(&self) -> Result<Vec<u8>, KeyManagerError> {
        self.get_bytes(DEFAULT_KEY_MANAGER_ATTESTATION_API)
    }

    pub fn export_keys_for_recipient_quote(
        &self,
        recipient_quote: &[u8],
    ) -> Result<KeyManagerExport, KeyManagerError> {
        let response = self.post_bytes(DEFAULT_KEY_MANAGER_EXPORT_API, recipient_quote)?;
        decode_key_manager_export_response(&response).map_err(|err| {
            KeyManagerError::Service(format!("decode export response: {err}"))
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn get_bytes(&self, path: &str) -> Result<Vec<u8>, KeyManagerError> {
        let response = self
            .agent
            .get(&self.endpoint(path))
            .call()
            .map_err(http_error)?;
        decode_base64_response(response)
    }

    fn post_bytes(&self, path: &str, body: &[u8]) -> Result<Vec<u8>, KeyManagerError> {
        let encoded = URL_SAFE.encode(body);
        let response = self
            .agent
            .post(&self.endpoint(path))
            .set("content-type", "text/plain")
            .send_string(&encoded)
            .map_err(http_error)?;
        decode_base64_response(response)
    }
}

pub fn encode_key_manager_export_response(export: &KeyManagerExport) -> Vec<u8> {
    let mut out = Vec::new();
    append_blob(&mut out, &export.key_manager_quote);
    append_blob(&mut out, &export.recipient_evidence);
    append_blob(&mut out, &export.envelope);
    out
}

pub fn decode_key_manager_export_response(payload: &[u8]) -> Result<KeyManagerExport, String> {
    let mut remaining = payload;
    let key_manager_quote = take_blob(&mut remaining)?.to_vec();
    let recipient_evidence = take_blob(&mut remaining)?.to_vec();
    let envelope = take_blob(&mut remaining)?.to_vec();

    if !remaining.is_empty() {
        return Err("trailing bytes after key-manager export response".to_string());
    }

    Ok(KeyManagerExport {
        key_manager_quote,
        recipient_evidence,
        envelope,
    })
}

fn decode_base64_response(response: ureq::Response) -> Result<Vec<u8>, KeyManagerError> {
    let body = response
        .into_string()
        .map_err(|err| KeyManagerError::Service(format!("read response body: {err}")))?;
    URL_SAFE
        .decode(body.trim())
        .map_err(|err| KeyManagerError::Service(format!("decode base64 response: {err}")))
}

fn http_error(err: ureq::Error) -> KeyManagerError {
    match err {
        ureq::Error::Status(code, response) => {
            let body = response
                .into_string()
                .unwrap_or_else(|err| format!("failed to read error body: {err}"));
            KeyManagerError::Service(format!("HTTP {code}: {body}"))
        }
        ureq::Error::Transport(err) => KeyManagerError::Service(err.to_string()),
    }
}
