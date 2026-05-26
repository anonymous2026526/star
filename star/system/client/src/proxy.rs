use openssl::ssl::{ExtensionContext, SslConnector, SslMethod, SslVerifyMode};
use pingora::protocols::tls::SslStream;
use pingora::protocols::Stream;
use pingora::server::Server;
use pingora::services::listening::Service;
use std::io;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::auth::StarAuthenticator;
use crate::config::ProxyConfig;
use crate::utils::{
    find_header_end, io_error, io_other, read_header, read_header_from,
};
use crate::{BoxError, STAR_TLS_EXTENSION_TYPE};

pub struct StarReverseProxy {
    upstream_host: String,
    upstream_port: u16,
    upstream_sni: String,
    authenticator: StarAuthenticator,
    insecure_tls: bool,
}

impl StarReverseProxy {
    pub fn new(config: ProxyConfig) -> Result<Self, BoxError> {
        let authenticator =
            StarAuthenticator::register(config.api_base.clone(), config.max_count)?;

        Ok(Self {
            upstream_host: config.upstream_host,
            upstream_port: config.upstream_port,
            upstream_sni: config.upstream_sni,
            authenticator,
            insecure_tls: config.insecure_tls,
        })
    }

    pub async fn handle(&self, downstream: &mut Stream) -> io::Result<()> {
        let request_head = read_header(downstream).await?;

        if is_preflight(&request_head) {
            (&mut **downstream)
                .write_all(cors_preflight_response().as_bytes())
                .await?;
            (&mut **downstream).flush().await?;
            (&mut **downstream).shutdown().await;
            return Ok(());
        }

        let token = self
            .authenticator
            .token_request()
            .map_err(|e| io_error(e.to_string()))?;

        let mut upstream = self.connect_upstream(token).await?;

        let request_head = force_connection_close(&request_head);

        upstream.write_all(&request_head).await?;
        upstream.flush().await?;

        let (mut downstream_read, mut downstream_write) = tokio::io::split(&mut **downstream);
        let (mut upstream_read, mut upstream_write) = tokio::io::split(upstream);

        let upload = async {
            let _ = tokio::io::copy(&mut downstream_read, &mut upstream_write).await;
            let _ = upstream_write.shutdown().await;
        };

        let download = async {
            let response_head = read_header_from(&mut upstream_read).await?;
            let response_head = add_cors_headers(&response_head);

            downstream_write.write_all(&response_head).await?;
            tokio::io::copy(&mut upstream_read, &mut downstream_write).await?;
            let _ = downstream_write.shutdown().await;

            Ok::<(), io::Error>(())
        };

        tokio::pin!(upload);
        tokio::pin!(download);

        tokio::select! {
            result = &mut download => {
                result?;
            }
            _ = &mut upload => {
                download.await?;
            }
        }

        Ok(())
    }

    async fn connect_upstream(&self, star_token: Vec<u8>) -> io::Result<SslStream<TcpStream>> {
        let tcp = TcpStream::connect((self.upstream_host.as_str(), self.upstream_port)).await?;

        let mut builder = SslConnector::builder(SslMethod::tls()).map_err(io_other)?;

        if self.insecure_tls {
            builder.set_verify(SslVerifyMode::NONE);
        }

        builder
            .add_custom_ext(
                STAR_TLS_EXTENSION_TYPE,
                ExtensionContext::TLS_ONLY
                    | ExtensionContext::CLIENT_HELLO
                    | ExtensionContext::TLS1_2_AND_BELOW_ONLY
                    | ExtensionContext::TLS1_3_ONLY,
                move |_, _, _| Ok(Some(star_token.clone())),
                |_, _, _, _| Ok(()),
            )
            .map_err(io_other)?;

        let ssl = builder
            .build()
            .configure()
            .map_err(io_other)?
            .into_ssl(&self.upstream_sni)
            .map_err(io_other)?;

        let mut tls = SslStream::new(ssl, tcp).map_err(io_other)?;
        tls.connect().await.map_err(io_other)?;

        Ok(tls)
    }
}

pub fn build_server(config: ProxyConfig) -> Result<Server, BoxError> {
    let listen_addr = config.listen_addr.clone();
    let proxy = StarReverseProxy::new(config)?;

    let mut server = Server::new(None)?;
    server.bootstrap();

    let mut service = Service::new("STAR reverse proxy".to_string(), proxy);
    service.add_tcp(&listen_addr);
    server.add_service(service);

    Ok(server)
}

fn is_preflight(request: &[u8]) -> bool {
    if !request.starts_with(b"OPTIONS ") {
        return false;
    }

    let Ok(head) = std::str::from_utf8(request) else {
        return false;
    };

    head.lines().any(|line| {
        line.to_ascii_lowercase()
            .starts_with("access-control-request-method:")
    })
}

fn cors_preflight_response() -> &'static str {
    "HTTP/1.1 204 No Content\r\n\
     Access-Control-Allow-Origin: *\r\n\
     Access-Control-Allow-Methods: GET, POST, PUT, PATCH, DELETE, OPTIONS\r\n\
     Access-Control-Allow-Headers: *\r\n\
     Access-Control-Max-Age: 86400\r\n\
     Content-Length: 0\r\n\
     Connection: close\r\n\
     \r\n"
}

fn force_connection_close(request: &[u8]) -> Vec<u8> {
    rewrite_headers(
        request,
        &[
            "connection:",
            "proxy-connection:",
            "keep-alive:",
        ],
        &[
            "Connection: close",
        ],
    )
}

fn add_cors_headers(response: &[u8]) -> Vec<u8> {
    rewrite_headers(
        response,
        &[
            "connection:",
            "proxy-connection:",
            "keep-alive:",
            "access-control-allow-origin:",
            "access-control-allow-methods:",
            "access-control-allow-headers:",
            "access-control-allow-credentials:",
            "access-control-expose-headers:",
            "access-control-max-age:",
        ],
        &[
            "Access-Control-Allow-Origin: *",
            "Access-Control-Allow-Methods: GET, POST, PUT, PATCH, DELETE, OPTIONS",
            "Access-Control-Allow-Headers: *",
            "Access-Control-Expose-Headers: *",
            "Connection: close",
        ],
    )
}

fn rewrite_headers(buf: &[u8], remove: &[&str], add: &[&str]) -> Vec<u8> {
    let Some(end) = find_header_end(buf) else {
        return buf.to_vec();
    };

    let head = &buf[..end - 4];
    let body = &buf[end..];

    let Ok(head) = std::str::from_utf8(head) else {
        return buf.to_vec();
    };

    let mut out = Vec::with_capacity(buf.len() + 256);

    for (i, line) in head.split("\r\n").enumerate() {
        if i != 0 {
            let lower = line.to_ascii_lowercase();

            if remove.iter().any(|name| lower.starts_with(name)) {
                continue;
            }
        }

        out.extend_from_slice(line.as_bytes());
        out.extend_from_slice(b"\r\n");
    }

    for line in add {
        out.extend_from_slice(line.as_bytes());
        out.extend_from_slice(b"\r\n");
    }

    out.extend_from_slice(b"\r\n");
    out.extend_from_slice(body);

    out
}