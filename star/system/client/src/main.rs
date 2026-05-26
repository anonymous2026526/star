use star_client_proxy::{config::ProxyConfig, proxy::build_server};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();
    let server = build_server(ProxyConfig::from_env())?;
    server.run_forever()
}
