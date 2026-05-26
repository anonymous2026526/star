use crate::bytes::Short;

pub fn current_period() -> Short{
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before UNIX_EPOCH");
    let period_secs = now.as_secs() / 60;
    period_secs.to_be_bytes()
}
