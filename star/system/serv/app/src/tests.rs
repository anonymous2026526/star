use crate::api::{DEFAULT_PUBLIC_KEY_API, DEFAULT_REGISTER_API};
use crate::enclave::report_data_for_public_key;
use crate::KeyManagerRuntime;
use crate::{all_prefix, build_server_without_startup_attestation};

use crate::tls::STAR_TLS_EXTENSION_TYPE;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use constant_time_utils::time;
use openssl::ssl::{ExtensionContext, SslConnector, SslMethod, SslVerifyMode};
use pingora::server::{RunArgs, ShutdownSignal, ShutdownSignalWatch};
use rand_core::OsRng;
use star_core::client::User;
use star_core::secure_channel::client::SecureChannelClient;
use star_tools::{QuoteVerificationPolicy, decode_quote_input};
use std::ffi::{OsStr, OsString};
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tiny_http::{Response, Server as TinyServer};
use tokio::sync::{Mutex as AsyncMutex, oneshot};

static SGX_TEST_LOCK: StdMutex<()> = StdMutex::new(());
static KEY_MANAGER_SEALED_KEY_COUNTER: AtomicUsize = AtomicUsize::new(0);
const DEFAULT_AESM_SOCKET: &str = "/run/aesmd/aesm.socket";

struct TestShutdown {
    rx: AsyncMutex<oneshot::Receiver<()>>,
}

#[async_trait]
impl ShutdownSignalWatch for TestShutdown {
    async fn recv(&self) -> ShutdownSignal {
        let mut rx = self.rx.lock().await;
        let _ = (&mut *rx).await;
        ShutdownSignal::FastShutdown
    }
}

fn retry_request<T, F>(mut op: F) -> T
where
    F: FnMut() -> Result<T, ureq::Error>,
{
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match op() {
            Ok(value) => return value,
            Err(ureq::Error::Status(code, _)) => panic!("http request failed: status {code}"),
            Err(ureq::Error::Transport(err)) => {
                if Instant::now() >= deadline {
                    panic!("http request failed after retrying: {err}");
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

fn get_text_retry(url: &str) -> String {
    retry_request(|| Ok(ureq::get(url).call()?.into_string()?))
}

fn post_text_retry(url: &str, body: &[u8]) -> String {
    retry_request(|| Ok(ureq::post(url).send_bytes(body)?.into_string()?))
}

fn https_get_with_star_extension(
    proxy_addr: SocketAddr,
    token_request: &[u8],
) -> Result<(u16, String), String> {
    let mut tls = SslConnector::builder(SslMethod::tls()).map_err(|e| e.to_string())?;
    tls.set_verify(SslVerifyMode::NONE);

    let ext = token_request.to_vec();
    tls.add_custom_ext(
        STAR_TLS_EXTENSION_TYPE,
        ExtensionContext::TLS_ONLY
            | ExtensionContext::CLIENT_HELLO
            | ExtensionContext::TLS1_2_AND_BELOW_ONLY
            | ExtensionContext::TLS1_3_ONLY,
        move |_, _, _| Ok(Some(ext.clone())),
        |_, _, _, _| Ok(()),
    )
    .map_err(|e| e.to_string())?;

    let tls = tls.build();
    https_get_with_connector(proxy_addr, tls)
}

fn https_get_without_star_extension(proxy_addr: SocketAddr) -> Result<(u16, String), String> {
    let mut tls = SslConnector::builder(SslMethod::tls()).map_err(|e| e.to_string())?;
    tls.set_verify(SslVerifyMode::NONE);

    let tls = tls.build();
    https_get_with_connector(proxy_addr, tls)
}

fn https_get_with_connector(
    proxy_addr: SocketAddr,
    tls: openssl::ssl::SslConnector,
) -> Result<(u16, String), String> {
    let stream = TcpStream::connect_timeout(&proxy_addr, Duration::from_secs(2))
        .map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;

    let mut stream = tls
        .connect("localhost", stream)
        .map_err(|e| format!("tls connect failed: {e}"))?;

    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).map_err(|e| e.to_string())?;
    let raw = String::from_utf8_lossy(&raw);
    let (head, body) = raw
        .split_once("\r\n\r\n")
        .ok_or_else(|| "invalid HTTP response".to_string())?;
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or_else(|| "missing HTTP status".to_string())?
        .parse::<u16>()
        .map_err(|e| e.to_string())?;

    Ok((status, body.to_string()))
}

fn unused_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    listener
        .local_addr()
        .expect("read ephemeral address")
        .to_string()
}

fn spawn_upstream() -> (String, thread::JoinHandle<()>) {
    let server = TinyServer::http("127.0.0.1:0").expect("bind upstream");
    let addr = server
        .server_addr()
        .to_ip()
        .expect("upstream should bind to IP");
    let addr = addr.to_string();
    let handle = thread::spawn(move || {
        if let Ok(request) = server.recv() {
            let response = Response::from_string("OK");
            let _ = request.respond(response);
        }
    });
    (addr, handle)
}

struct EnvVarGuard {
    name: &'static str,
    previous: Option<OsString>,
}

impl EnvVarGuard {
    fn set(name: &'static str, value: impl AsRef<OsStr>) -> Self {
        let previous = std::env::var_os(name);
        unsafe {
            std::env::set_var(name, value);
        }
        Self { name, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(previous) = &self.previous {
                std::env::set_var(self.name, previous);
            } else {
                std::env::remove_var(self.name);
            }
        }
    }
}

struct KeyManagerSealedKeysGuard {
    _env: EnvVarGuard,
    path: PathBuf,
}

impl KeyManagerSealedKeysGuard {
    fn fresh() -> Self {
        let count = KEY_MANAGER_SEALED_KEY_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "star-key-manager-test-{}-{count}.sealed",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let env = EnvVarGuard::set("STAR_KEY_MANAGER_SEALED_KEYS", path.as_os_str());

        Self { _env: env, path }
    }
}

impl Drop for KeyManagerSealedKeysGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

struct AesmProxy {
    addr: String,
    shutdown: Option<mpsc::Sender<()>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl AesmProxy {
    fn start_on(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let addr = listener.local_addr()?.to_string();
        let socket_path = aesm_socket_path();
        let (shutdown, shutdown_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            loop {
                if shutdown_rx.try_recv().is_ok() {
                    break;
                }

                match listener.accept() {
                    Ok((stream, _)) => {
                        let socket_path = socket_path.clone();
                        thread::spawn(move || {
                            if let Err(err) = forward_aesm_connection(stream, socket_path) {
                                eprintln!("AESM proxy connection failed: {err}");
                            }
                        });
                    }
                    Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => panic!("accept AESM proxy connection: {err}"),
                }
            }
        });

        Ok(Self {
            addr,
            shutdown: Some(shutdown),
            handle: Some(handle),
        })
    }
}

impl Drop for AesmProxy {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn start_aesm_proxy_for_sgx_tests() -> (Option<AesmProxy>, EnvVarGuard) {
    let aesm_proxy = match AesmProxy::start_on("127.0.0.1:5555") {
        Ok(proxy) => Some(proxy),
        Err(err) if err.kind() == io::ErrorKind::AddrInUse => None,
        Err(err) => panic!("start AESM proxy: {err}"),
    };
    let aesm_proxy_addr = aesm_proxy
        .as_ref()
        .map(|proxy| proxy.addr.clone())
        .unwrap_or_else(|| "127.0.0.1:5555".to_string());
    let aesm_proxy_env = EnvVarGuard::set("AESM_PROXY", aesm_proxy_addr);

    (aesm_proxy, aesm_proxy_env)
}

fn aesm_socket_path() -> PathBuf {
    std::env::var_os("AESM_SOCKET")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_AESM_SOCKET))
}

#[cfg(unix)]
fn forward_aesm_connection(mut tcp: TcpStream, socket_path: PathBuf) -> io::Result<()> {
    loop {
        let Some((request_len, request)) = read_aesm_frame(&mut tcp)? else {
            return Ok(());
        };

        let mut unix = UnixStream::connect(&socket_path)?;
        unix.write_all(&request_len.to_ne_bytes())?;
        unix.write_all(&request)?;

        let Some((response_len, response)) = read_aesm_frame(&mut unix)? else {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "AESM closed before sending a response",
            ));
        };
        tcp.write_all(&response_len.to_ne_bytes())?;
        tcp.write_all(&response)?;
    }
}

fn read_aesm_frame(stream: &mut impl Read) -> io::Result<Option<(u32, Vec<u8>)>> {
    let mut len = [0u8; 4];
    match stream.read_exact(&mut len) {
        Ok(()) => {}
        Err(err)
            if matches!(
                err.kind(),
                io::ErrorKind::UnexpectedEof
                    | io::ErrorKind::ConnectionReset
                    | io::ErrorKind::BrokenPipe
            ) =>
        {
            return Ok(None);
        }
        Err(err) => return Err(err),
    }

    let len = u32::from_ne_bytes(len);
    if len > 16 * 1024 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("AESM frame too large: {len} bytes"),
        ));
    }

    let mut body = vec![0u8; len as usize];
    stream.read_exact(&mut body)?;
    Ok(Some((len, body)))
}

#[test]
fn test_all() {
    let _sgx_test_guard = SGX_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (_aesm_proxy, _aesm_proxy_env) = start_aesm_proxy_for_sgx_tests();
    let _sealed_keys = KeyManagerSealedKeysGuard::fresh();
    let api_addr = unused_loopback_addr();
    let proxy_addr = unused_loopback_addr();
    let (upstream_addr, upstream_handle) = spawn_upstream();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let api_bind_addr = api_addr.clone();
    let proxy_bind_addr = proxy_addr.clone();
    let tls_cert_path = format!("{}/examples/certs/server.crt", env!("CARGO_MANIFEST_DIR"));
    let tls_key_path = format!("{}/examples/certs/server.key", env!("CARGO_MANIFEST_DIR"));

    unsafe {
        std::env::set_var("TLS_CERT_PATH", tls_cert_path.as_str());
        std::env::set_var("TLS_KEY_PATH", tls_key_path.as_str());
        std::env::set_var("STAR_ALLOW_DEBUG_ENCLAVES", "1");
        std::env::set_var("STAR_ALLOW_ADVISORY_ENCLAVES", "1");
        std::env::set_var("STAR_TRUST_SAME_SIGNER_ENCLAVES", "1");
    }

    let server_handle = thread::spawn(move || {
        let server = build_server_without_startup_attestation(
            &api_bind_addr,
            &proxy_bind_addr,
            &upstream_addr,
            all_prefix(1),
            1,
        )
        .expect("build test server");
        let shutdown = TestShutdown {
            rx: AsyncMutex::new(shutdown_rx),
        };
        let run_args = RunArgs {
            shutdown_signal: Box::new(shutdown),
        };
        server.run(run_args);
    });

    thread::sleep(Duration::from_millis(100));

    let public_key_url = format!("http://{api_addr}{DEFAULT_PUBLIC_KEY_API}");
    let body = get_text_retry(&public_key_url);
    let public_key = URL_SAFE.decode(body.trim()).unwrap();
    let public_key: [u8; 32] = public_key.as_slice().try_into().unwrap();
    println!("{:?}", public_key);

    let client = SecureChannelClient::new(public_key).expect("client init");
    let mut user = User::new(1, client);
    let mut rng = OsRng;

    let (cred_request, shared_key) = user.request_credential(&mut rng);
    let cred_request_b64 = URL_SAFE.encode(cred_request);

    println!("{cred_request_b64}");

    let register_url = format!("http://{api_addr}{DEFAULT_REGISTER_API}");
    let body = post_text_retry(&register_url, cred_request_b64.as_bytes());
    let cred_cipher = URL_SAFE.decode(body.trim()).unwrap();
    user.receive_credential(&cred_cipher, &shared_key);

    let current_period = u64::from_be_bytes(time::current_period());
    let token_request: Vec<u8> = user
        .request_auth(0, current_period, &mut rng)
        .expect("token request");

    let proxy_socket = proxy_addr
        .to_socket_addrs()
        .expect("resolve proxy address")
        .next()
        .expect("proxy address should resolve");

    let (status, body) =
        https_get_with_star_extension(proxy_socket, &token_request).expect("proxy request");
    assert_eq!(status, 200);
    assert_eq!(body.trim(), "OK");

    match https_get_with_star_extension(proxy_socket, &token_request) {
        Ok((status, _)) => panic!("expected handshake rejection for duplicate token, got {status}"),
        Err(_) => {}
    }

    match https_get_with_star_extension(proxy_socket, &[]) {
        Ok((status, _)) => panic!("expected handshake rejection for empty token, got {status}"),
        Err(_) => {}
    }

    match https_get_without_star_extension(proxy_socket) {
        Ok((status, _)) => {
            panic!("expected handshake rejection when STAR extension is missing, got {status}")
        }
        Err(_) => {}
    }

    let _ = shutdown_tx.send(());
    let _ = server_handle.join();
    let _ = upstream_handle.join();
}

#[test]
fn public_key_attestation_quote_verifies_with_tools_policy() {
    let _sgx_test_guard = SGX_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (_aesm_proxy, _aesm_proxy_env) = start_aesm_proxy_for_sgx_tests();
    let _sealed_keys = KeyManagerSealedKeysGuard::fresh();

    let key_manager = KeyManagerRuntime::from_env().expect("initialize key-manager");
    let public_key = key_manager
        .public_key()
        .expect("read key-manager public key");
    let quote = key_manager
        .public_key_attestation()
        .expect("get public key attestation quote");

    let startup_log = format!(
        "STAR_ATTESTATION_PUBLIC_KEY_BASE64={}\nSTAR_ATTESTATION_QUOTE_BASE64={}\n",
        URL_SAFE.encode(public_key),
        URL_SAFE.encode(&quote)
    );
    let quote = decode_quote_input(startup_log.as_bytes()).expect("decode quote from startup log");
    let parsed = star_tools::parse_quote(&quote).expect("parse quote");
    let expected_mrsigner = parsed.mrsigner;
    let expected_mrenclave = parsed.mrenclave;

    assert_eq!(
        star_tools::public_key_from_report_data(&parsed).expect("extract attested public key"),
        public_key
    );
    assert_eq!(parsed.report_data, report_data_for_public_key(public_key));

    let policy = QuoteVerificationPolicy {
        expected_mrsigner: &expected_mrsigner,
        expected_mrenclave: Some(&expected_mrenclave),
        expected_public_key: Some(&public_key),
        allow_debug: true,
        allow_advisory: true,
        pccs_url: None,
    };
    let verified = star_tools::verify_quote(&quote, &policy).expect("verify quote with tools");

    assert_eq!(verified.public_key, public_key);
    assert_eq!(
        verified.parsed.report_data,
        report_data_for_public_key(public_key)
    );
}
