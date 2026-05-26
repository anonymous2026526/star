use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use rand::rngs::OsRng;
use star_core::{client::User, secure_channel::client::SecureChannelClient};
use std::sync::{Arc, Mutex};
use crate::{BoxError, utils::{current_period, http_get_text, http_post_text, io_error, io_other}};

#[derive(Clone)]
pub struct StarAuthenticator {
    api_base: String,
    max_count: u64,
    inner: Arc<Mutex<StarAuthState>>,
}

struct StarAuthState {
    user: User,
    counter: u64,
    period: u64,
}

impl StarAuthenticator {
    pub fn register(api_base: String, max_count: u64) -> Result<Self, BoxError> {
        let public_key = http_get_text(&format!(
            "{}/star/public_key",
            api_base.trim_end_matches('/')
        ))?;

        let public_key = URL_SAFE.decode(public_key.trim())?;
        let public_key: [u8; 32] = public_key
            .as_slice()
            .try_into()
            .map_err(|_| "public key must be 32 bytes")?;

        let client = SecureChannelClient::new(public_key)
            .map_err(|e| format!("failed to initialize secure channel: {e:?}"))?;

        let mut user = User::new(max_count, client);
        let mut rng = OsRng;

        let (request, shared_key) = user.request_credential(&mut rng);
        let request = URL_SAFE.encode(request);

        let credential_cipher = http_post_text(
            &format!("{}/star/register", api_base.trim_end_matches('/')),
            request.as_bytes(),
        )?;

        let credential_cipher = URL_SAFE.decode(credential_cipher.trim())?;

        user.receive_credential(&credential_cipher, &shared_key)
            .ok_or("failed to receive credential")?;

        Ok(Self {
            api_base,
            max_count,
            inner: Arc::new(Mutex::new(StarAuthState {
                user,
                counter: 0,
                period: current_period(),
            })),
        })
    }

    pub fn token_request(&self) -> Result<Vec<u8>, BoxError> {
        let mut state = self.inner.lock().map_err(|_| "STAR auth mutex poisoned")?;
        let period = current_period();

        if state.period != period {
            state.period = period;
            state.counter = 0;
        }

        if state.counter >= self.max_count {
            return Err(format!(
                "exceeded {max} STAR token requests for period {period}; wait for the next minute",
                max = self.max_count
            )
            .into());
        }

        let counter = state.counter;
        state.counter += 1;

        let mut rng = OsRng;

        state
            .user
            .request_auth(counter, period, &mut rng)
            .map_err(|e| {
                format!(
                    "failed to build STAR token request via {}: {e:?}",
                    self.api_base
                )
                .into()
            })
    }
}
