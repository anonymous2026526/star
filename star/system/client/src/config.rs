use std::env;

use crate::utils;

#[derive(Clone, Debug)]
pub struct ProxyConfig {
    pub listen_addr: String,
    pub api_base: String,
    pub upstream_host: String,
    pub upstream_port: u16,
    pub upstream_sni: String,
    pub insecure_tls: bool,
    pub max_count: u64,
}

impl ProxyConfig {
    pub fn from_env() -> Self {
        let upstream_host =
            env::var("STAR_UPSTREAM_HOST").unwrap_or_else(|_| "localhost".to_string());

        let upstream_port = env::var("STAR_UPSTREAM_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(8081);

        let upstream_sni = env::var("STAR_UPSTREAM_SNI").unwrap_or_else(|_| upstream_host.clone());

        Self {
            listen_addr: env::var("STAR_CLIENT_PROXY_LISTEN")
                .unwrap_or_else(|_| "127.0.0.1:18080".to_string()),
            api_base: env::var("STAR_API_BASE")
                .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string()),
            upstream_host,
            upstream_port,
            upstream_sni,
            insecure_tls: true, //utils::truthy_env("STAR_TLS_INSECURE"),
            max_count: env::var("STAR_MAX_COUNT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(10),
        }
    }
}
