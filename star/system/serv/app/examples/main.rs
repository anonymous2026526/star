use star_app::{all_prefix, build_server};
use std::env;
use std::thread;
use tiny_http::{Header, Response, Server as TinyServer};

const HTML_TEMPLATE: &str = include_str!("html_template.html");

fn render_html(title: &str, headline: &str, body: &str) -> String {
    HTML_TEMPLATE
        .replace("{{title}}", title)
        .replace("{{headline}}", headline)
        .replace("{{body}}", body)
}

fn spawn_content_server(bind_addr: &str) -> (String, thread::JoinHandle<()>) {
    let server = TinyServer::http(bind_addr).expect("bind content server");
    let addr = server
        .server_addr()
        .to_ip()
        .expect("content server should bind to IP")
        .to_string();
    let handle = thread::spawn(move || {
        while let Ok(request) = server.recv() {
            let body = render_html(
                "tiny_http HTML",
                "Hello from tiny_http",
                "This response is HTML.",
            );
            let mut response = Response::from_string(body);
            let header = Header::from_bytes(b"Content-Type", b"text/html; charset=utf-8")
                .expect("build content-type header");
            response.add_header(header);
            let _ = request.respond(response);
        }
    });
    (addr, handle)
}

fn main() {
    let api_addr = env_or_default("STAR_API_ADDR", "0.0.0.0:8080");
    let proxy_addr = env_or_default("STAR_PROXY_ADDR", "0.0.0.0:18081");
    let content_bind_addr = env_or_first(
        &[
            "STAR_EXAMPLE_CONTENT_ADDR",
            "STAR_EXAMPLE_HTML_ADDR",
            "STAR_EXAMPLE_UPSTREAM_ADDR",
        ],
        "127.0.0.1:3000",
    );

    let (content_addr, _content_handle) = spawn_content_server(&content_bind_addr);
    let server = build_server(&api_addr, &proxy_addr, &content_addr, all_prefix(1), 1);
    server.unwrap().run_forever();
}

fn env_or_default(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn env_or_first(names: &[&str], default: &str) -> String {
    names
        .iter()
        .find_map(|name| env::var(name).ok())
        .unwrap_or_else(|| default.to_string())
}
