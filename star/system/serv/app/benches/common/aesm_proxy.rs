use std::ffi::{OsStr, OsString};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const DEFAULT_AESM_PROXY: &str = "127.0.0.1:5555";
const DEFAULT_AESM_SOCKET: &str = "/run/aesmd/aesm.socket";

pub struct AesmProxyGuard {
    _proxy: Option<AesmProxy>,
    _env: Option<EnvVarGuard>,
}

impl AesmProxyGuard {
    pub fn start() -> Self {
        if std::env::var_os("AESM_PROXY").is_some() {
            return Self {
                _proxy: None,
                _env: None,
            };
        }

        let proxy = match AesmProxy::start_on(DEFAULT_AESM_PROXY) {
            Ok(proxy) => Some(proxy),
            Err(err) if err.kind() == io::ErrorKind::AddrInUse => None,
            Err(err) => panic!("start AESM proxy: {err}"),
        };
        let addr = proxy
            .as_ref()
            .map(|proxy| proxy.addr.clone())
            .unwrap_or_else(|| DEFAULT_AESM_PROXY.to_string());
        let env = EnvVarGuard::set("AESM_PROXY", addr);

        Self {
            _proxy: proxy,
            _env: Some(env),
        }
    }
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

#[cfg(not(unix))]
fn forward_aesm_connection(_: TcpStream, _: PathBuf) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "AESM proxy requires Unix sockets",
    ))
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
