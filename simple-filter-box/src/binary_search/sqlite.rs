use crate::core::{FilterBox, bytes::Long};
use rusqlite::{Connection, OptionalExtension};
use std::{io, path::Path};

const CREATE_TABLE_SQL: &str = r#"
    create table if not exists dedup_tokens (
        token blob primary key
    )
"#;

pub struct SqliteFilter {
    conn: Connection,
}

impl SqliteFilter {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let conn =
            Connection::open(path).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        conn.execute(CREATE_TABLE_SQL, [])
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        Ok(Self { conn })
    }
}

impl FilterBox for SqliteFilter {
    fn from_entries(_tokens: &[Long]) -> io::Result<Self> {
        todo!();
    }

    fn insert(&self, token: &Long) -> io::Result<()> {
        self.conn
            .execute(
                "insert or ignore into dedup_tokens (token) values (?1)",
                [&token[..]],
            )
            .map_err(|err| {
                eprintln!("failed to insert sqlite dedup token: {err}");
                io::Error::new(io::ErrorKind::Other, err)
            })?;
        Ok(())
    }

    fn test(&self, token: &Long) -> bool {
        match self
            .conn
            .query_row(
                "select 1 from dedup_tokens where token = ?1 limit 1",
                [&token[..]],
                |_| Ok(()),
            )
            .optional()
        {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(err) => {
                eprintln!("failed to read sqlite dedup table: {err}");
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after epoch")
            .as_nanos();
        path.push(format!(
            "simple_filter_box_test_{}_{}.sqlite",
            std::process::id(),
            nanos
        ));
        path
    }

    #[test]
    fn insert_and_test_roundtrip() {
        let path = temp_db_path();
        let mut filter = SqliteFilter::open(&path).expect("open sqlite");

        let token1: Long = [3u8; 32];
        let token2: Long = [4u8; 32];

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

    #[test]
    fn reopen_preserves_tokens() {
        let path = temp_db_path();
        {
            let mut filter = SqliteFilter::open(&path).expect("open sqlite");
            let token: Long = [8u8; 32];
            assert!(filter.test(&token));
            filter.insert(&token).unwrap();
            assert!(!filter.test(&token));
        }

        let filter = SqliteFilter::open(&path).expect("reopen sqlite");
        let token: Long = [8u8; 32];
        assert!(!filter.test(&token));

        let _ = std::fs::remove_file(path);
    }
}
