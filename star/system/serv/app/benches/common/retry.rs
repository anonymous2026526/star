use std::thread;
use std::time::Duration;

pub enum StartupProbe {
    Ready,
    Retry(String),
    Fatal(String),
}

pub fn wait_until_ready<F>(service: &str, retries: u32, retry_delay: Duration, mut probe: F)
where
    F: FnMut() -> StartupProbe,
{
    let mut last_retry_reason = None;
    for _ in 0..retries {
        match probe() {
            StartupProbe::Ready => return,
            StartupProbe::Retry(reason) => {
                last_retry_reason = Some(reason);
                thread::sleep(retry_delay);
            }
            StartupProbe::Fatal(reason) => panic!("{service} failed to start cleanly: {reason}"),
        }
    }

    panic!(
        "{service} did not become ready within startup timeout, last retry reason: {:?}",
        last_retry_reason
    );
}
