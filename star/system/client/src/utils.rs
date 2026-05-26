use std::{env, fmt, io};

use pingora::protocols::Stream;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::{BoxError, MAX_HEADER_SIZE};

pub(crate) fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|pos| pos + 4)
}

pub(crate) fn current_period() -> u64 {
    u64::from_be_bytes(constant_time_utils::time::current_period())
}

pub(crate) fn http_get_text(url: &str) -> Result<String, BoxError> {
    Ok(ureq::get(url).call()?.into_string()?)
}

pub(crate) fn http_post_text(url: &str, body: &[u8]) -> Result<String, BoxError> {
    Ok(ureq::post(url).send_bytes(body)?.into_string()?)
}

pub(crate) fn truthy_env(name: &str) -> bool {
    env::var(name)
        .map(|value| {
            let value = value.trim();
            !value.is_empty() && value != "0" && !value.eq_ignore_ascii_case("false")
        })
        .unwrap_or(false)
}

pub(crate) fn io_error(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message.into())
}

pub(crate) fn io_other(error: impl fmt::Display) -> io::Error {
    io_error(error.to_string())
}

pub async fn read_header(stream: &mut Stream) -> io::Result<Vec<u8>> {
    read_header_from(stream).await
}

pub async fn read_header_from<R>(stream: &mut R) -> io::Result<Vec<u8>>
where
    R: AsyncRead + Unpin,
{
    let mut out = Vec::with_capacity(4096);
    let mut buf = [0u8; 2048];

    loop {
        let n = stream.read(&mut buf).await?;

        if n == 0 {
            return Err(io_error("connection closed before header"));
        }

        out.extend_from_slice(&buf[..n]);

        if find_header_end(&out).is_some() {
            return Ok(out);
        }

        if out.len() > MAX_HEADER_SIZE {
            return Err(io_error("header too large"));
        }
    }
}