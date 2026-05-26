use foreign_types::ForeignTypeRef;
use openssl::error::ErrorStack;
use openssl::ex_data::Index;
use openssl::ssl::{ClientHelloResponse, Ssl};
use pingora::listeners::tls::TlsSettings;
use pingora::tls::ssl::{ExtensionContext, SslAlert, SslRef};
use simple_filter_box::core::FilterBox;
use std::os::raw::{c_int, c_uchar, c_uint};
use std::sync::{Arc, OnceLock};
use std::{ptr, slice};

use crate::filter::StarFilter;
static STAR_TLS_TOKEN_EX_INDEX: OnceLock<Index<Ssl, Vec<u8>>> = OnceLock::new();
static STAR_TLS_DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

use crate::tls_utils;

pub const STAR_TLS_EXTENSION_TYPE: u16 = 65280;

fn star_tls_debug_enabled() -> bool {
    *STAR_TLS_DEBUG_ENABLED.get_or_init(|| {
        std::env::var("STAR_TLS_DEBUG")
            .map(|v| {
                let v = v.trim();
                !v.is_empty() && v != "0" && !v.eq_ignore_ascii_case("false")
            })
            .unwrap_or(false)
    })
}

macro_rules! star_tls_debug {
    ($($arg:tt)*) => {
        if star_tls_debug_enabled() {
            eprintln!("[star-tls] {}", format_args!($($arg)*));
        }
    };
}

unsafe extern "C" {
    fn SSL_client_hello_get0_ext(
        s: *mut openssl_sys::SSL,
        type_: c_uint,
        out: *mut *const c_uchar,
        outlen: *mut usize,
    ) -> c_int;
}

fn token_ex_index() -> Result<Index<Ssl, Vec<u8>>, openssl::error::ErrorStack> {
    if let Some(index) = STAR_TLS_TOKEN_EX_INDEX.get() {
        star_tls_debug!("reusing OpenSSL ex_data index");
        return Ok(*index);
    }

    star_tls_debug!("creating OpenSSL ex_data index");

    let index = Ssl::new_ex_index::<Vec<u8>>()?;
    let _ = STAR_TLS_TOKEN_EX_INDEX.set(index);

    star_tls_debug!("created OpenSSL ex_data index");

    Ok(*STAR_TLS_TOKEN_EX_INDEX.get().expect("TLS token index set"))
}

fn star_tls_extension_from_client_hello<'a>(ssl: &'a SslRef) -> Option<&'a [u8]> {
    let mut out = ptr::null();
    let mut out_len = 0usize;

    let found = unsafe {
        SSL_client_hello_get0_ext(
            ssl.as_ptr(),
            STAR_TLS_EXTENSION_TYPE as c_uint,
            &mut out,
            &mut out_len,
        )
    };

    if found != 1 {
        return None;
    }

    if out_len == 0 {
        return Some(&[]);
    }

    if out.is_null() {
        return None;
    }

    Some(unsafe { slice::from_raw_parts(out as *const u8, out_len) })
}

fn reject_client_hello(
    alert: &mut SslAlert,
    reason: &'static str,
) -> Result<ClientHelloResponse, ErrorStack> {
    star_tls_debug!(
        "rejecting ClientHello: reason={}, extension_type={}",
        reason,
        STAR_TLS_EXTENSION_TYPE
    );

    *alert = SslAlert::DECODE_ERROR;
    Err(ErrorStack::get())
}

pub fn configure_tls_token_extension<T>(
    tls_settings: &mut TlsSettings,
    filter: Arc<StarFilter<T>>,
) -> Result<(), openssl::error::ErrorStack>
where
    T: FilterBox + Send + Sync + 'static,
{
    tls_settings.set_groups_list("X25519")?;

    star_tls_debug!(
        "configuring custom extension: extension_type={}",
        STAR_TLS_EXTENSION_TYPE
    );

    let token_index = token_ex_index()?;

    tls_settings.set_client_hello_callback(|ssl, alert| {
        let ext_data = match star_tls_extension_from_client_hello(ssl) {
            Some(ext_data) => ext_data,
            None => {
                return reject_client_hello(alert, "missing_custom_extension");
            }
        };

        if ext_data.is_empty() {
            return reject_client_hello(alert, "empty_custom_extension");
        }

        star_tls_debug!(
            "accepted ClientHello custom extension presence check: extension_type={}, ext_data_len={}",
            STAR_TLS_EXTENSION_TYPE,
            ext_data.len()
        );

        Ok(ClientHelloResponse::SUCCESS)
    });

    tls_settings.add_custom_ext(
        STAR_TLS_EXTENSION_TYPE,
        ExtensionContext::CLIENT_HELLO,
        |_ssl, _context, _cert| {
            star_tls_debug!(
                "custom extension add callback called: extension_type={}",
                STAR_TLS_EXTENSION_TYPE
            );

            Ok(None::<Vec<u8>>)
        },
        move |ssl, _context, ext_data, _cert| {
            star_tls_debug!(
                "custom extension parse callback called: extension_type={}, ext_data_len={}",
                STAR_TLS_EXTENSION_TYPE,
                ext_data.len()
            );

            if ext_data.is_empty() {
                star_tls_debug!(
                    "rejecting custom extension: reason=empty_ext_data, extension_type={}",
                    STAR_TLS_EXTENSION_TYPE
                );

                return Err(SslAlert::DECODE_ERROR);
            }

            if let Some(existing) = ssl.ex_data(token_index) {
                if existing.as_slice() == ext_data {
                    star_tls_debug!(
                        "custom extension parsed again with identical data: extension_type={}, ext_data_len={}",
                        STAR_TLS_EXTENSION_TYPE,
                        ext_data.len()
                    );

                    return Ok(());
                }

                star_tls_debug!(
                    "rejecting custom extension: reason=different_duplicate_ext_data, extension_type={}, existing_len={}, new_len={}",
                    STAR_TLS_EXTENSION_TYPE,
                    existing.len(),
                    ext_data.len()
                );

                return Err(SslAlert::ILLEGAL_PARAMETER);
            }

            if let Err(filter_err) = filter.filter_client_hello(ext_data) {
                let alert = tls_utils::alert_for_filter_error(&filter_err);

                star_tls_debug!(
                    "rejecting custom extension: reason=filter_rejected, extension_type={}, ext_data_len={}, filter_error={}, alert={}",
                    STAR_TLS_EXTENSION_TYPE,
                    ext_data.len(),
                    tls_utils::filter_error_name(&filter_err),
                    tls_utils::alert_name(alert)
                );

                return Err(alert);
            }

            ssl.set_ex_data(token_index, ext_data.to_vec());

            star_tls_debug!(
                "stored custom extension data in ex_data: extension_type={}, ext_data_len={}",
                STAR_TLS_EXTENSION_TYPE,
                ext_data.len()
            );

            Ok(())
        },
    )?;

    star_tls_debug!(
        "configured custom extension: extension_type={}",
        STAR_TLS_EXTENSION_TYPE
    );

    Ok(())
}
