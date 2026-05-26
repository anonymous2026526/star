use std::io;

use crate::core::bytes::Long;

pub type Entry = [u8; 32];

pub mod bytes {
    use super::Entry;

    pub type Long = Entry;
}

pub enum FilterResult {
    Exist,
    NotExist,
}

pub trait FilterBox {
    fn from_entries(token: &[Long]) -> io::Result<Self>
    where
        Self: Sized;
    fn insert(&self, token: &Entry) -> io::Result<()>;
    fn test(&self, token: &Entry) -> bool;

    fn test_and_insert(&self, token: &Entry) -> io::Result<bool> {
        let is_new = self.test(token);
        if is_new {
            self.insert(token)?;
        }
        Ok(is_new)
    }
}
