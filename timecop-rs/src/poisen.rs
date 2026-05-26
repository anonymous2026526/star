
use crabgrind::memcheck;

pub fn poison(key: &[u8]) {
    match memcheck::mark_memory(
        key.as_ptr().cast(),
        key.len(),
        memcheck::MemState::Undefined,
    ) {
        Ok(()) => {}
        Err(_) => panic!("failed to poison key region"),
    }
}

pub fn unpoison(key: &[u8]) {
    match memcheck::mark_memory(
        key.as_ptr().cast(),
        key.len(),
        memcheck::MemState::Defined,
    ) {
        Ok(()) => {}
        Err(_) => panic!("failed to unpoison key region"),
    }
}
