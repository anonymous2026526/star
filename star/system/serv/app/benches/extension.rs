mod common;

use std::io::{self, Read, Write};
use std::os::raw::{c_int, c_uchar, c_uint};
use std::ptr;
use std::slice;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use foreign_types::ForeignTypeRef;
use openssl::error::ErrorStack;
use openssl::ssl::{
    ClientHelloResponse, ExtensionContext, HandshakeError, Ssl, SslAcceptor, SslAlert,
    SslConnector, SslFiletype, SslMethod, SslRef, SslVerifyMode,
};
use rand_core::OsRng;
use simple_filter_box::{core::Entry, dashmap::DashMapFilterBox};
use star_app::enclave::{EnclaveRuntime, EnclaveRuntimes};
use star_app::filter::{StarFilter, StarFilterErrorStatus};
use star_app::{
    KeyManagerRuntime, install_key_manager_keys_for_enclave, install_key_manager_keys_for_pool,
};
use star_app::tls::STAR_TLS_EXTENSION_TYPE;
use star_core::{client::User, secure_channel::client::SecureChannelClient};

const TLS_CERT_RELATIVE_PATH: &str = "examples/certs/server.crt";
const TLS_KEY_RELATIVE_PATH: &str = "examples/certs/server.key";
const MAX_COUNT_PER_CREDENTIAL: u64 = 1_000_000;
const FILTER_ENCLAVE_POOL_SIZE: u8 = 10;
const PARALLEL_WORKERS: usize = 2;
const PARALLEL_OPS_PER_WORKER: u64 = 64;
const PARALLEL_BATCH_OPS: u64 = PARALLEL_WORKERS as u64 * PARALLEL_OPS_PER_WORKER;

type BenchFilter = StarFilter<DashMapFilterBox>;

unsafe extern "C" {
    fn SSL_client_hello_get0_ext(
        s: *mut openssl_sys::SSL,
        type_: c_uint,
        out: *mut *const c_uchar,
        outlen: *mut usize,
    ) -> c_int;
}

#[derive(Debug)]
struct ClientHelloRecorder {
    output: Vec<u8>,
}

impl ClientHelloRecorder {
    fn new() -> Self {
        Self { output: Vec::new() }
    }
}

impl Read for ClientHelloRecorder {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::from(io::ErrorKind::WouldBlock))
    }
}

impl Write for ClientHelloRecorder {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct ClientHelloInput<'a> {
    input: &'a [u8],
    read_offset: usize,
}

impl<'a> ClientHelloInput<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            read_offset: 0,
        }
    }
}

impl Read for ClientHelloInput<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.read_offset >= self.input.len() {
            return Err(io::Error::from(io::ErrorKind::WouldBlock));
        }

        let remaining = &self.input[self.read_offset..];
        let len = remaining.len().min(buf.len());
        buf[..len].copy_from_slice(&remaining[..len]);
        self.read_offset += len;
        Ok(len)
    }
}

impl Write for ClientHelloInput<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ClientHelloOutcome {
    PendingMoreInput,
    Rejected,
}

// This is for benchmarking the case with without STAR extension in ClientHello
struct OpenSslRejectHarness {
    acceptor: SslAcceptor,
    client_hello: Vec<u8>,
}

// This is for benchmarking the case with STAR extension in ClientHello but rejected by filter (e.g. replayed token, invalid credential)
struct StarFilterRejectHarness {
    acceptor: SslAcceptor,
    filter: Arc<BenchFilter>,
    token_request: Vec<u8>,
    client_hello: Vec<u8>,
    duplicate_token: Option<Entry>,
    duplicate_route: u32,
}

impl OpenSslRejectHarness {
    fn missing_extension() -> Self {
        let mut builder = base_acceptor_builder();

        builder.set_client_hello_callback(move |ssl, alert| {
            if star_extension_from_client_hello(ssl).is_none() {
                return reject_client_hello(alert, SslAlert::DECODE_ERROR);
            }

            Ok(ClientHelloResponse::SUCCESS)
        });

        let connector = connector_with_star_extension(None);

        Self {
            acceptor: builder.build(),
            client_hello: record_client_hello(&connector),
        }
    }

    fn timed_rejections(&self, iters: u64) -> Duration {
        let mut total = Duration::ZERO;

        for _ in 0..iters {
            let start = Instant::now();
            let outcome =
                process_client_hello(&self.acceptor, black_box(self.client_hello.as_slice()));
            total += start.elapsed();
            assert_eq!(outcome, ClientHelloOutcome::Rejected);
        }

        total
    }

    fn timed_parallel_rejections(&self, iters: u64) -> Duration {
        timed_parallel_batch(iters, || {
            let outcome =
                process_client_hello(&self.acceptor, black_box(self.client_hello.as_slice()));
            assert_eq!(outcome, ClientHelloOutcome::Rejected);
        })
    }
}

impl StarFilterRejectHarness {
    // Initilize the benchmark with a replayed token.
    fn replayed_token() -> Self {
        let (filter, token_request) = build_keyed_filter_and_token_request();
        let (duplicate_token, duplicate_route) = filter
            .issue_token_and_route_sync(&token_request)
            .expect("issue duplicate token");
        filter
            .check_duplication(&duplicate_token, duplicate_route)
            .expect("seed duplicate token");
        let harness = Self::new(
            filter,
            token_request,
            Some(duplicate_token),
            duplicate_route,
        );

        let outcome = process_client_hello(&harness.acceptor, harness.client_hello.as_slice());
        assert_eq!(outcome, ClientHelloOutcome::Rejected);

        harness
    }


    // Initialize the benchmark with an invalid credential (e.g. corrupted token).
    fn invalid_credential() -> Self {
        let (filter, token_request) = build_keyed_filter_and_invalid_credential_request();
        let harness = Self::new(filter, token_request, None, 0);

        assert_eq!(
            process_client_hello(&harness.acceptor, harness.client_hello.as_slice()),
            ClientHelloOutcome::Rejected
        );
        assert!(matches!(
            harness
                .filter
                .filter_client_hello(harness.token_request.as_slice()),
            Err(StarFilterErrorStatus::InvalidToken)
        ));

        harness
    }

    fn new(
        filter: Arc<BenchFilter>,
        token_request: Vec<u8>,
        duplicate_token: Option<Entry>,
        duplicate_route: u32,
    ) -> Self {
        let connector = connector_with_star_extension(Some(token_request.clone()));
        let client_hello = record_client_hello(&connector);
        let mut builder = base_acceptor_builder();
        let token_index = Ssl::new_ex_index::<Vec<u8>>().expect("TLS token ex_data index");
        let callback_filter = Arc::clone(&filter);

        builder.set_client_hello_callback(move |ssl, alert| {
            let Some(ext_data) = star_extension_from_client_hello(ssl) else {
                return reject_client_hello(alert, SslAlert::DECODE_ERROR);
            };

            if ext_data.is_empty() {
                return reject_client_hello(alert, SslAlert::DECODE_ERROR);
            }

            Ok(ClientHelloResponse::SUCCESS)
        });


        // This callback is for measuring the combined cost of OpenSSL parsing the ClientHello and the filter rejecting it. 
        // It simulates the real flow where the filter checks the token during the ClientHello callback and rejects it if invalid (e.g. replayed or invalid credential).
        builder
            .add_custom_ext(
                STAR_TLS_EXTENSION_TYPE,
                ExtensionContext::CLIENT_HELLO,
                |_ssl, _context, _cert| Ok(None::<Vec<u8>>),
                move |ssl, _context, ext_data, _cert| {
                    if ext_data.is_empty() {
                        return Err(SslAlert::DECODE_ERROR);
                    }

                    if let Some(existing) = ssl.ex_data(token_index) {
                        if existing.as_slice() == ext_data {
                            return Ok(());
                        }

                        return Err(SslAlert::ILLEGAL_PARAMETER);
                    }

                    if let Err(filter_err) = callback_filter.filter_client_hello(ext_data) {
                        return Err(alert_for_filter_error(&filter_err));
                    }

                    ssl.set_ex_data(token_index, ext_data.to_vec());
                    Ok(())
                },
            )
            .expect("configure STAR TLS extension");

        Self {
            acceptor: builder.build(),
            filter,
            token_request,
            client_hello,
            duplicate_token,
            duplicate_route,
        }
    }

    fn timed_openssl_and_filter_rejections(&self, iters: u64) -> Duration {
        let mut total = Duration::ZERO;

        for _ in 0..iters {
            let start = Instant::now();
            let outcome =
                process_client_hello(&self.acceptor, black_box(self.client_hello.as_slice()));
            total += start.elapsed();
            assert_eq!(outcome, ClientHelloOutcome::Rejected);
        }

        total
    }

    fn timed_parallel_openssl_and_filter_rejections(&self, iters: u64) -> Duration {
        timed_parallel_batch(iters, || {
            let outcome =
                process_client_hello(&self.acceptor, black_box(self.client_hello.as_slice()));
            assert_eq!(outcome, ClientHelloOutcome::Rejected);
        })
    }
}

fn timed_parallel_batch(operation_batches: u64, operation: impl Fn() + Sync) -> Duration {
    let total_ops = operation_batches
        .checked_mul(PARALLEL_BATCH_OPS)
        .expect("parallel benchmark operation count overflow");
    let ready_barrier = Barrier::new(PARALLEL_WORKERS + 1);
    let start_barrier = Barrier::new(PARALLEL_WORKERS + 1);

    thread::scope(|scope| {
        let operation = &operation;
        let ready_barrier = &ready_barrier;
        let start_barrier = &start_barrier;
        let mut handles = Vec::with_capacity(PARALLEL_WORKERS);

        for worker_index in 0..PARALLEL_WORKERS {
            let worker_ops = worker_operation_count(total_ops, worker_index);
            handles.push(scope.spawn(move || {
                ready_barrier.wait();
                start_barrier.wait();

                for _ in 0..worker_ops {
                    operation();
                }
            }));
        }

        ready_barrier.wait();
        let start = Instant::now();
        start_barrier.wait();

        for handle in handles {
            handle.join().expect("parallel benchmark worker");
        }

        start.elapsed()
    })
}

fn worker_operation_count(total_ops: u64, worker_index: usize) -> u64 {
    let workers = PARALLEL_WORKERS as u64;
    let base = total_ops / workers;
    let remainder = total_ops % workers;

    base + u64::from((worker_index as u64) < remainder)
}

// This runs OpenSSL, processing the ClientHello.
fn process_client_hello(acceptor: &SslAcceptor, client_hello: &[u8]) -> ClientHelloOutcome {
    match acceptor.accept(ClientHelloInput::new(client_hello)) {
        Ok(_) => ClientHelloOutcome::PendingMoreInput,
        Err(HandshakeError::WouldBlock(_)) => ClientHelloOutcome::PendingMoreInput,
        Err(HandshakeError::Failure(_)) => ClientHelloOutcome::Rejected,
        Err(HandshakeError::SetupFailure(err)) => panic!("TLS server setup failed: {err}"),
    }
}

fn record_client_hello(connector: &SslConnector) -> Vec<u8> {
    let recorder = ClientHelloRecorder::new();

    let client_hello = match connector.connect("localhost", recorder) {
        Ok(_) => panic!("client handshake unexpectedly completed without server input"),
        Err(HandshakeError::WouldBlock(mid)) | Err(HandshakeError::Failure(mid)) => {
            mid.get_ref().output.clone()
        }
        Err(HandshakeError::SetupFailure(err)) => panic!("TLS client setup failed: {err}"),
    };

    assert!(!client_hello.is_empty(), "recorded ClientHello is empty");
    client_hello
}

fn build_keyed_filter_and_token_request() -> (Arc<BenchFilter>, Vec<u8>) {
    build_keyed_filter_and_request(build_token_request)
}

fn build_keyed_filter_and_invalid_credential_request() -> (Arc<BenchFilter>, Vec<u8>) {
    build_keyed_filter_and_request(build_invalid_credential_token_request)
}

fn build_keyed_filter_and_request(
    build_request: impl FnOnce(&EnclaveRuntime) -> Vec<u8>,
) -> (Arc<BenchFilter>, Vec<u8>) {
    let _aesm_proxy = common::aesm_proxy::AesmProxyGuard::start();
    let key_manager = KeyManagerRuntime::from_env_with_sealed_keys(&[]).expect("key manager");
    let filter_enclave =
        EnclaveRuntimes::from_env(FILTER_ENCLAVE_POOL_SIZE).expect("filter enclave pool");
    let issue_enclave = EnclaveRuntime::from_env().expect("issue enclave");

    install_key_manager_keys_for_pool(&key_manager, &filter_enclave)
        .expect("install filter enclave benchmark keys");
    install_key_manager_keys_for_enclave(&key_manager, &issue_enclave)
        .expect("install issue enclave benchmark keys");

    let filter = Arc::new(BenchFilter::new(
        DashMapFilterBox::new(),
        vec![0],
        1,
        filter_enclave,
    ));
    let token_request = build_request(&issue_enclave);

    (filter, token_request)
}

fn build_token_request(issue_enclave: &EnclaveRuntime) -> Vec<u8> {
    let sender =
        SecureChannelClient::new(issue_enclave.public_key().expect("public key")).expect("sender");
    let mut user = User::new(MAX_COUNT_PER_CREDENTIAL, sender);
    let mut rng = OsRng;

    let (cred_request, shared_key) = user.request_credential(&mut rng);
    let cred_cipher = issue_enclave
        .issue_cred(&cred_request)
        .expect("issue credential");
    user.receive_credential(&cred_cipher, &shared_key);

    let period = u64::from_be_bytes(constant_time_utils::time::current_period());
    user.request_auth(0, period, &mut rng)
        .expect("token request")
}

fn build_invalid_credential_token_request(issue_enclave: &EnclaveRuntime) -> Vec<u8> {
    let sender =
        SecureChannelClient::new(issue_enclave.public_key().expect("public key")).expect("sender");
    let mut user = User::new(MAX_COUNT_PER_CREDENTIAL, sender);
    let mut rng = OsRng;

    let (cred_request, shared_key) = user.request_credential(&mut rng);
    let cred_cipher = issue_enclave
        .issue_cred(&cred_request)
        .expect("issue credential");
    user.receive_credential(&cred_cipher, &shared_key);

    let credential = user.credential.as_mut().expect("credential");
    credential.code[0] ^= 0x01;

    let period = u64::from_be_bytes(constant_time_utils::time::current_period());
    user.request_auth(0, period, &mut rng)
        .expect("token request")
}

fn base_acceptor_builder() -> openssl::ssl::SslAcceptorBuilder {
    let mut builder =
        SslAcceptor::mozilla_intermediate_v5(SslMethod::tls()).expect("build TLS acceptor");
    builder
        .set_certificate_file(manifest_path(TLS_CERT_RELATIVE_PATH), SslFiletype::PEM)
        .expect("set TLS certificate");
    builder
        .set_private_key_file(manifest_path(TLS_KEY_RELATIVE_PATH), SslFiletype::PEM)
        .expect("set TLS private key");
    builder.check_private_key().expect("check TLS private key");
    builder.set_groups_list("X25519").expect("set TLS groups");
    builder
}

fn connector_with_star_extension(ext: Option<Vec<u8>>) -> SslConnector {
    let mut builder = SslConnector::builder(SslMethod::tls()).expect("build TLS connector");
    builder.set_verify(SslVerifyMode::NONE);
    builder.set_groups_list("X25519").expect("set TLS groups");

    if let Some(ext) = ext {
        builder
            .add_custom_ext(
                STAR_TLS_EXTENSION_TYPE,
                ExtensionContext::CLIENT_HELLO,
                move |_ssl, _context, _cert| Ok(Some(ext.clone())),
                |_ssl, _context, _ext_data, _cert| Ok(()),
            )
            .expect("configure client STAR extension");
    }

    builder.build()
}

fn reject_client_hello(
    alert: &mut SslAlert,
    ssl_alert: SslAlert,
) -> Result<ClientHelloResponse, ErrorStack> {
    *alert = ssl_alert;
    Err(ErrorStack::get())
}

fn star_extension_from_client_hello<'a>(ssl: &'a SslRef) -> Option<&'a [u8]> {
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

fn alert_for_filter_error(status: &StarFilterErrorStatus) -> SslAlert {
    match status {
        StarFilterErrorStatus::InvalidToken => SslAlert::DECODE_ERROR,
        StarFilterErrorStatus::InvalidRoute => SslAlert::ILLEGAL_PARAMETER,
        StarFilterErrorStatus::DuplicatedToken => SslAlert::ILLEGAL_PARAMETER,
        StarFilterErrorStatus::Internal => SslAlert::DECODE_ERROR,
    }
}

fn manifest_path(relative_path: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), relative_path)
}

fn configure_group(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    group.sample_size(20);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(4));
    group.throughput(Throughput::Elements(1));
}

fn configure_parallel_group(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
) {
    group.sample_size(20);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(4));
    group.throughput(Throughput::Elements(PARALLEL_BATCH_OPS));
}

fn bench_client_hello_reject(c: &mut Criterion) {
    let openssl = OpenSslRejectHarness::missing_extension();
    let star_filter = StarFilterRejectHarness::replayed_token();
    let invalid_credential = StarFilterRejectHarness::invalid_credential();

    assert_eq!(
        process_client_hello(&openssl.acceptor, openssl.client_hello.as_slice()),
        ClientHelloOutcome::Rejected
    );
    assert_eq!(
        process_client_hello(&star_filter.acceptor, star_filter.client_hello.as_slice()),
        ClientHelloOutcome::Rejected
    );
    let duplicate_token = star_filter
        .duplicate_token
        .as_ref()
        .expect("duplicate token");
    assert!(matches!(
        star_filter
            .filter
            .check_duplication(duplicate_token, star_filter.duplicate_route),
        Err(StarFilterErrorStatus::DuplicatedToken)
    ));
    assert_eq!(
        process_client_hello(
            &invalid_credential.acceptor,
            invalid_credential.client_hello.as_slice()
        ),
        ClientHelloOutcome::Rejected
    );
    assert!(matches!(
        invalid_credential
            .filter
            .filter_client_hello(invalid_credential.token_request.as_slice()),
        Err(StarFilterErrorStatus::InvalidToken)
    ));

    let mut group = c.benchmark_group("client_hello_reject");
    configure_group(&mut group);

    group.bench_function("openssl_missing_extension", |b| {
        b.iter_custom(|iters| openssl.timed_rejections(iters));
    });

    group.bench_function("star_filter_invalid_credential_openssl_parse", |b| {
        b.iter_custom(|iters| invalid_credential.timed_openssl_and_filter_rejections(iters));
    });

    group.bench_function("star_filter_replayed_token_openssl_parse", |b| {
        b.iter_custom(|iters| star_filter.timed_openssl_and_filter_rejections(iters));
    });

    group.finish();

    let mut group = c.benchmark_group("client_hello_reject_parallel_throughput");
    configure_parallel_group(&mut group);

    group.bench_function("openssl_missing_extension", |b| {
        b.iter_custom(|iters| openssl.timed_parallel_rejections(iters));
    });

    group.bench_function("star_filter_replayed_token_openssl_parse", |b| {
        b.iter_custom(|iters| star_filter.timed_parallel_openssl_and_filter_rejections(iters));
    });

    group.bench_function("star_filter_invalid_credential_openssl_parse", |b| {
        b.iter_custom(|iters| {
            invalid_credential.timed_parallel_openssl_and_filter_rejections(iters)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_client_hello_reject);
criterion_main!(benches);
