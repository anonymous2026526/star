pub const DEFAULT_REGISTER_API: &str = "/star/register";
pub const DEFAULT_PUBLIC_KEY_API: &str = "/star/public_key";

use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::URL_SAFE};

use constant_time_utils::bytes::Long;
use pingora_web::{Handler, PingoraHttpRequest, PingoraWebHttpResponse, StatusCode, WebError};
use std::sync::Arc;

use crate::enclave::{EnclaveError, EnclaveRuntime};

pub struct IssueApi {
    enclave: Arc<EnclaveRuntime>,
}

impl IssueApi {
    pub fn init(enclave: EnclaveRuntime) -> Self {
        let enclave = Arc::new(enclave);
        Self::init_shared(enclave)
    }

    pub fn init_shared(enclave: Arc<EnclaveRuntime>) -> Self {
        IssueApi { enclave }
    }
}

#[async_trait]
impl Handler for IssueApi {
    async fn handle(&self, req: PingoraHttpRequest) -> Result<PingoraWebHttpResponse, WebError> {
        let body = req.body().as_ref();
        let decoded = match URL_SAFE.decode(body) {
            Ok(decoded) => decoded,
            Err(_) => {
                return Ok(PingoraWebHttpResponse::text(StatusCode::BAD_REQUEST, ""));
            }
        };

        let cred = match self.enclave.issue_cred(&decoded) {
            Ok(cred) => cred,
            Err(_) => {
                return Ok(PingoraWebHttpResponse::text(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "",
                ));
            }
        };
        let out = URL_SAFE.encode(cred);
        let res = PingoraWebHttpResponse::text(StatusCode::OK, out);
        Ok(res)
    }
}

pub struct PublicKeyApi {
    public_key: Long,
}

impl PublicKeyApi {
    pub fn init(enclave: &mut EnclaveRuntime) -> Result<Self, EnclaveError> {
        let public_key = enclave.public_key()?;

        Ok(Self::from_public_key(public_key))
    }

    pub fn from_public_key(public_key: Long) -> Self {
        PublicKeyApi { public_key }
    }
}

#[async_trait]
impl Handler for PublicKeyApi {
    async fn handle(&self, _req: PingoraHttpRequest) -> Result<PingoraWebHttpResponse, WebError> {
        let out = URL_SAFE.encode(self.public_key);
        Ok(PingoraWebHttpResponse::text(StatusCode::OK, out))
    }
}
