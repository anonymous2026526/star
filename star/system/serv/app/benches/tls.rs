mod common;

use common::local_http::reserve_local_addr;
use common::retry::{wait_until_ready, StartupProbe};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use openssl::ssl::{SslAcceptor, SslConnector, SslFiletype, SslMethod, SslVerifyMode};
use std::convert::TryFrom;
use std::error::Error;
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const STARTUP_RETRIES: u32 = 100;
const STARTUP_RETRY_DELAY: Duration = Duration::from_millis(50);
const WARMUP_REQUESTS: usize = 128;
const TLS_CERT_RELATIVE_PATH: &str = "examples/certs/server.crt";
const TLS_KEY_RELATIVE_PATH: &str = "examples/certs/server.key";
const IO_TIMEOUT: Duration = Duration::from_secs(2);
const LATENCY_RECV_TIMEOUT: Duration = Duration::from_secs(2);
const BENCH_GROUP_NAME: &str = "pure_tls_server_side_latency";
const TLS_HANDSHAKE_ONLY: &str = "server_side_tls_handshake";

fn cert_path() -> String {
    common::manifest_path(TLS_CERT_RELATIVE_PATH)
}

fn key_path() -> String {
    common::manifest_path(TLS_KEY_RELATIVE_PATH)
}

fn nanos_to_duration(ns: u128) -> Duration {
    Duration::from_nanos(u64::try_from(ns).unwrap_or(u64::MAX))
}

fn elapsed_ns(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn build_acceptor(cert_pem_path: &str, key_pem_path: &str) -> SslAcceptor {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())
        .expect("build TLS acceptor");
    builder
        .set_private_key_file(key_pem_path, SslFiletype::PEM)
        .expect("set server TLS private key");
    builder
        .set_certificate_chain_file(cert_pem_path)
        .expect("set server TLS certificate chain");
    builder
        .set_groups_list("X25519")
        .expect("set server TLS group");
    builder.build()
}

fn spawn_tls_server(
    bind_addr: String,
    cert_pem_path: String,
    key_pem_path: String,
    latency_tx: mpsc::Sender<u64>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let acceptor = build_acceptor(&cert_pem_path, &key_pem_path);
        let listener = TcpListener::bind(&bind_addr).expect("bind TLS benchmark server");

        for tcp in listener.incoming() {
            let tcp = tcp.expect("accept TCP connection");
            tcp.set_read_timeout(Some(IO_TIMEOUT))
                .expect("set TCP read timeout");
            tcp.set_write_timeout(Some(IO_TIMEOUT))
                .expect("set TCP write timeout");

            // Server-side TLS handshake only.
            // Start: after TCP accept, immediately before OpenSSL accepts the TLS handshake.
            // End: immediately after OpenSSL finishes the TLS handshake successfully.
            let start = Instant::now();
            let _tls = acceptor.accept(tcp).expect("accept TLS handshake");
            let _ = latency_tx.send(elapsed_ns(start));
        }
    })
}

struct PlainTlsClient {
    connector: SslConnector,
}

impl PlainTlsClient {
    fn new() -> Self {
        let mut builder = SslConnector::builder(SslMethod::tls()).expect("build TLS connector");
        builder.set_verify(SslVerifyMode::NONE);
        builder
            .set_groups_list("X25519")
            .expect("set client TLS group");

        Self {
            connector: builder.build(),
        }
    }

    fn handshake_only(&self, addr: SocketAddr) -> Result<(), Box<dyn Error + Send + Sync>> {
        let tcp = TcpStream::connect(addr)?;
        tcp.set_read_timeout(Some(IO_TIMEOUT))?;
        tcp.set_write_timeout(Some(IO_TIMEOUT))?;

        let _tls = self.connector.connect("localhost", tcp)?;
        Ok(())
    }
}

struct BenchHarness {
    server_addr: SocketAddr,
    tls_client: PlainTlsClient,
    latency_rx: mpsc::Receiver<u64>,
    _server_handle: thread::JoinHandle<()>,
}

impl BenchHarness {
    fn start() -> Self {
        let cert_pem_path = cert_path();
        let key_pem_path = key_path();
        let server_addr = reserve_local_addr();
        let (latency_tx, latency_rx) = mpsc::channel();

        let server_handle = spawn_tls_server(
            server_addr.clone(),
            cert_pem_path,
            key_pem_path,
            latency_tx,
        );

        let server_addr = server_addr
            .to_socket_addrs()
            .expect("resolve server address")
            .next()
            .expect("server address should resolve");

        let mut harness = Self {
            server_addr,
            tls_client: PlainTlsClient::new(),
            latency_rx,
            _server_handle: server_handle,
        };

        harness.wait_until_ready();
        harness.warmup(WARMUP_REQUESTS);
        harness.drain_latencies();
        harness
    }

    fn run_client_once(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.tls_client.handshake_only(self.server_addr)
    }

    fn one_server_side_latency_ns(&mut self) -> u64 {
        self.run_client_once().expect("run TLS client once");
        self.latency_rx
            .recv_timeout(LATENCY_RECV_TIMEOUT)
            .expect("wait server-side latency")
    }

    fn wait_until_ready(&mut self) {
        wait_until_ready(
            "pure TLS benchmark server",
            STARTUP_RETRIES,
            STARTUP_RETRY_DELAY,
            || match self.run_client_once() {
                Ok(()) => {
                    let _ = self.latency_rx.recv_timeout(LATENCY_RECV_TIMEOUT);
                    StartupProbe::Ready
                }
                Err(err) => StartupProbe::Retry(format!("startup transport error: {err}")),
            },
        );
    }

    fn warmup(&mut self, count: usize) {
        for _ in 0..count {
            let _ = self.one_server_side_latency_ns();
        }
    }

    fn drain_latencies(&self) {
        while self.latency_rx.try_recv().is_ok() {}
    }

    fn measure_server_side_total(&mut self, iters: u64) -> Duration {
        let mut total_ns = 0_u128;
        for _ in 0..iters {
            total_ns += u128::from(self.one_server_side_latency_ns());
        }
        nanos_to_duration(total_ns)
    }
}

fn bench_pure_tls_server_side_latency(c: &mut Criterion) {
    let mut harness = BenchHarness::start();

    let mut group = c.benchmark_group(BENCH_GROUP_NAME);
    group.sample_size(40);
    group.measurement_time(Duration::from_secs(6));
    group.throughput(Throughput::Elements(1));

    group.bench_function(TLS_HANDSHAKE_ONLY, |b| {
        b.iter_custom(|iters| harness.measure_server_side_total(iters));
    });

    group.finish();
}

criterion_group!(benches, bench_pure_tls_server_side_latency);
criterion_main!(benches);
