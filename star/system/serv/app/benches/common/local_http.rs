use std::net::TcpListener;

pub fn reserve_local_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve local port");
    let addr = listener.local_addr().expect("read reserved local port");
    drop(listener);
    addr.to_string()
}
