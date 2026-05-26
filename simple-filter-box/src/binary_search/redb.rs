use crate::core::{FilterBox, bytes::Long};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::{io, path::Path};

const TOKEN_TABLE: TableDefinition<&[u8], u8> = TableDefinition::new("dedup_tokens");

fn open_database(path: &Path) -> io::Result<Database> {
    let result = if path.exists() {
        Database::open(path)
    } else {
        Database::create(path)
    };
    result.map_err(|err| io::Error::new(io::ErrorKind::Other, err))
}

pub struct RedbFilter {
    db: Database,
}

impl RedbFilter {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let db = open_database(path.as_ref())?;
        let write_txn = db
            .begin_write()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        write_txn
            .open_table(TOKEN_TABLE)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        write_txn
            .commit()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        Ok(Self { db })
    }
}

impl FilterBox for RedbFilter {
    fn from_entries(_tokens: &[Long]) -> io::Result<Self> {
        todo!();
    }

    fn insert(&self, token: &Long) -> io::Result<()> {
        let write_txn = self.db.begin_write().map_err(|err| {
            eprintln!("failed to start redb write transaction: {err}");
            io::Error::new(io::ErrorKind::Other, err)
        })?;

        let mut table = write_txn.open_table(TOKEN_TABLE).map_err(|err| {
            eprintln!("failed to open redb dedup table: {err}");
            io::Error::new(io::ErrorKind::Other, err)
        })?;

        match table.get(&token[..]) {
            Ok(Some(_)) => return Ok(()),
            Ok(None) => {}
            Err(err) => {
                eprintln!("failed to read redb dedup table: {err}");
                return Err(io::Error::new(io::ErrorKind::Other, err));
            }
        }

        if let Err(err) = table.insert(&token[..], &1u8) {
            eprintln!("failed to insert redb dedup token: {err}");
            return Err(io::Error::new(io::ErrorKind::Other, err));
        }

        drop(table);
        if let Err(err) = write_txn.commit() {
            eprintln!("failed to commit redb dedup transaction: {err}");
            return Err(io::Error::new(io::ErrorKind::Other, err));
        }

        Ok(())
    }

    fn test(&self, token: &Long) -> bool {
        let read_txn = match self.db.begin_read() {
            Ok(txn) => txn,
            Err(err) => {
                eprintln!("failed to start redb read transaction: {err}");
                return false;
            }
        };

        let table = match read_txn.open_table(TOKEN_TABLE) {
            Ok(table) => table,
            Err(err) => {
                eprintln!("failed to open redb dedup table: {err}");
                return false;
            }
        };

        match table.get(&token[..]) {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(err) => {
                eprintln!("failed to read redb dedup table: {err}");
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after epoch")
            .as_nanos();
        path.push(format!(
            "simple_filter_box_test_{}_{}.redb",
            std::process::id(),
            nanos
        ));
        path
    }

    #[test]
    fn insert_and_test_roundtrip() {
        let path = temp_db_path();
        let mut filter = RedbFilter::open(&path).expect("open redb");

        let token1: Long = [1u8; 32];
        let token2: Long = [2u8; 32];

        assert!(filter.test(&token1));
        assert!(filter.test(&token2));

        filter.insert(&token1).unwrap();
        assert!(!filter.test(&token1));
        assert!(filter.test(&token2));

        filter.insert(&token1).unwrap();
        assert!(!filter.test(&token1));

        drop(filter);
        let _ = std::fs::remove_file(path);
    }
}
