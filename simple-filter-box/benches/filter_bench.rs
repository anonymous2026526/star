use criterion::{black_box, BenchmarkId, Criterion, criterion_group, criterion_main};
use redb::{Database, TableDefinition};
use rusqlite::{Connection, Transaction};
use simple_filter_box::binary_search::redb::RedbFilter;
use simple_filter_box::binary_search::sqlite::SqliteFilter;
use simple_filter_box::bloom_filter::BloomFilterBox;
use simple_filter_box::core::{FilterBox, bytes::Long};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::{io, path::Path};

const PRELOAD_COUNTS: &[usize] = &[
    1_000_000, 2_000_000, 3_000_000, 4_000_000, 
    5_000_000, 6_000_000, 7_000_000, 8_000_000, 9_000_000,
    // 100_000, 110_000, 120_000, 130_000, 140_000, 150_000, 160_000, 170_000, 180_000,
];

// const PRELOAD_COUNTS: &[usize] = &[
//     10_000, 20_000, 30_000, 40_000, 50_000, 60_000, 70_000, 80_000, 90_000,
//     100_000, 110_000, 120_000, 130_000, 140_000, 150_000, 160_000, 170_000, 180_000,
// ];
// const PRELOAD_COUNTS: &[usize] = &[100_000, 200_000, 300_000, 400_000, 500_000, 600_000, 700_000, 800_000, 900_000];
// const PRELOAD_COUNTS: &[usize] = &[100,200];
const TOKEN_TABLE: TableDefinition<&[u8], u8> = TableDefinition::new("dedup_tokens");
// const SAMPLE_SIZE: usize = 10;
const SAMPLE_SIZE: usize = 200;
const TARGET_TOKEN_COUNT: usize = 1000;

#[derive(Copy, Clone)]
enum DbBackend {
    Redb,
    Sqlite,
}

impl DbBackend {
    fn from_env() -> Self {
        match std::env::var("FILTER_BACKEND").ok().as_deref() {
            Some("sqlite") => DbBackend::Sqlite,
            Some("redb") | None => DbBackend::Redb,
            Some(other) => {
                eprintln!(
                    "unknown FILTER_BACKEND={other}, defaulting to redb (use 'redb' or 'sqlite')"
                );
                DbBackend::Redb
            }
        }
    }

    fn extension(self) -> &'static str {
        match self {
            DbBackend::Redb => "redb",
            DbBackend::Sqlite => "sqlite",
        }
    }
}

#[derive(Copy, Clone)]
enum BenchBackend {
    Bloom,
    Redb,
    Sqlite,
}

impl BenchBackend {
    fn label(self) -> &'static str {
        match self {
            BenchBackend::Bloom => "bloom",
            BenchBackend::Redb => "redb",
            BenchBackend::Sqlite => "sqlite",
        }
    }
}

impl From<DbBackend> for BenchBackend {
    fn from(value: DbBackend) -> Self {
        match value {
            DbBackend::Redb => BenchBackend::Redb,
            DbBackend::Sqlite => BenchBackend::Sqlite,
        }
    }
}

struct PreparedFilter {
    filter: Box<dyn FilterBox>,
    path: Option<PathBuf>,
}

impl PreparedFilter {
    fn into_parts(self) -> (Box<dyn FilterBox>, Option<PathBuf>) {
        (self.filter, self.path)
    }
}

fn make_tokens(count: usize, seed: u64) -> Vec<Long> {
    let mut tokens = Vec::with_capacity(count);
    let mut state = seed;
    for _ in 0..count {
        let mut token = [0u8; 32];
        for chunk in token.chunks_exact_mut(8) {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1);
            chunk.copy_from_slice(&state.to_le_bytes());
        }
        tokens.push(token);
    }
    tokens
}

fn temp_db_path(label: &str, extension: &str) -> PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let mut path = std::env::temp_dir();
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    path.push(format!(
        "simple_filter_box_bench_{}_{}_{}.{}",
        std::process::id(),
        label,
        id,
        extension
    ));
    path
}

fn open_database(path: &Path) -> io::Result<Database> {
    let result = if path.exists() {
        Database::open(path)
    } else {
        Database::create(path)
    };
    result.map_err(|err| io::Error::new(io::ErrorKind::Other, err))
}

fn insert_batch_redb(path: &Path, tokens: &[Long]) -> io::Result<()> {
    let db = open_database(path)?;
    let write_txn = db
        .begin_write()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    let mut table = write_txn
        .open_table(TOKEN_TABLE)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    for token in tokens {
        table
            .insert(&token[..], &1u8)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    }

    drop(table);
    write_txn
        .commit()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    Ok(())
}

fn open_sqlite(path: &Path) -> io::Result<Connection> {
    let conn = Connection::open(path).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    conn.execute(
        "create table if not exists dedup_tokens (token blob primary key)",
        [],
    )
    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    
    // avoid cache effect 
    conn.execute_batch(
        "PRAGMA cache_size = -2000;
         PRAGMA temp_store = MEMORY;
         PRAGMA mmap_size = 0;
         PRAGMA journal_mode = DELETE;
         PRAGMA synchronous = FULL;",
    )
    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    Ok(conn)
}

fn insert_batch_sqlite(path: &Path, tokens: &[Long]) -> io::Result<()> {
    let mut conn = open_sqlite(path)?;
    let tx = conn
        .transaction()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    insert_batch_sqlite_tx(&tx, tokens)?;
    tx.commit()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    Ok(())
}

fn insert_batch_sqlite_tx(tx: &Transaction<'_>, tokens: &[Long]) -> io::Result<()> {
    let mut stmt = tx
        .prepare("insert or ignore into dedup_tokens (token) values (?1)")
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    for token in tokens {
        stmt.execute([&token[..]])
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    }
    Ok(())
}

fn bench_backends() -> Vec<BenchBackend> {
    let db_backend = BenchBackend::from(DbBackend::from_env());
    match std::env::var("FILTER_BENCH").ok().as_deref() {
        Some("bloom") => vec![BenchBackend::Bloom],
        Some("redb") => vec![BenchBackend::Redb],
        Some("sqlite") => vec![BenchBackend::Sqlite],
        Some(other) => {
            eprintln!(
                "unknown FILTER_BENCH={other}, defaulting to bloom + {} (use 'bloom', 'redb', or 'sqlite')",
                db_backend.label()
            );
            vec![BenchBackend::Bloom, db_backend]
        }
        None => vec![BenchBackend::Bloom, db_backend],
    }
}

fn prepare_filter(
    backend: BenchBackend,
    preload: &[Long],
    label: &str,
) -> io::Result<PreparedFilter> {
    match backend {
        BenchBackend::Bloom => {
            let mut filter = BloomFilterBox::with_rate(preload.len() + 1, 0.01);
            for token in preload {
                filter.insert(token)?;
            }
            Ok(PreparedFilter {
                filter: Box::new(filter),
                path: None,
            })
        }
        BenchBackend::Redb => {
            let path = temp_db_path(label, DbBackend::Redb.extension());
            insert_batch_redb(&path, preload)?;
            let filter = RedbFilter::open(&path)?;
            Ok(PreparedFilter {
                filter: Box::new(filter),
                path: Some(path),
            })
        }
        BenchBackend::Sqlite => {
            let path = temp_db_path(label, DbBackend::Sqlite.extension());
            insert_batch_sqlite(&path, preload)?;
            let filter = SqliteFilter::open(&path)?;
            Ok(PreparedFilter {
                filter: Box::new(filter),
                path: Some(path),
            })
        }
    }
}

fn cleanup_path(path: Option<PathBuf>) {
    if let Some(path) = path {
        let _ = std::fs::remove_file(path);
    }
}

fn bench_insert(c: &mut Criterion) {
    let backends = bench_backends();
    let mut group = c.benchmark_group("insert");
    group.sample_size(SAMPLE_SIZE);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    for &count in PRELOAD_COUNTS {
        let preload = make_tokens(count, 0x5EED_u64);
        let insert_tokens = make_tokens(TARGET_TOKEN_COUNT, 0xBEE5_u64);

        for backend in &backends {
            group.bench_with_input(BenchmarkId::new(backend.label(), count), &count, |b, &_count| {
                let backend = *backend;
                b.iter_custom(|iters| {
                    let mut total = Duration::ZERO;
                    let mut insert_index = 0usize;
                    for _ in 0..iters {
                        let (mut filter, path) =
                            prepare_filter(backend, &preload, "insert").unwrap().into_parts();

                        let token = &insert_tokens[insert_index % insert_tokens.len()];
                        insert_index = insert_index.wrapping_add(1);
                        let start = Instant::now();
                        filter.insert(black_box(token)).unwrap();
                        total += start.elapsed();

                        drop(filter);
                        cleanup_path(path);
                    }
                    total
                });
            });
        }
    }

    group.finish();
}

fn bench_test(c: &mut Criterion) {
    let backends = bench_backends();
    let mut group = c.benchmark_group("test");
    group.sample_size(SAMPLE_SIZE);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    for &count in PRELOAD_COUNTS {
        let existing = make_tokens(count, 0xBADC0DE_u64);
        let missing = make_tokens(TARGET_TOKEN_COUNT, 0xD15EA5E_u64);

        for backend in &backends {
            group.bench_with_input(
                BenchmarkId::new(format!("{}_miss", backend.label()), count),
                &count,
                |b, &_count| {
                    let backend = *backend;
                    let (filter, path) =
                        prepare_filter(backend, &existing, "test").unwrap().into_parts();
                    let mut filter = filter;
                    let mut miss_index = 0usize;
                    b.iter(|| {
                        let token = &missing[miss_index % missing.len()];
                        miss_index = miss_index.wrapping_add(1);
                        black_box(filter.test(black_box(token)));
                    });
                    drop(filter);
                    cleanup_path(path);
                },
            );
        }
    }

    group.finish();
}

#[cfg(feature = "bench-insert-and-test")]
fn bench_insert_and_test(c: &mut Criterion) {
    let backends = bench_backends();
    let mut group = c.benchmark_group("insert_and_test");
    group.sample_size(SAMPLE_SIZE);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    for &count in PRELOAD_COUNTS {
        let preload = make_tokens(count, 0xABAD1DEA_u64);
        let insert_tokens = make_tokens(TARGET_TOKEN_COUNT, 0x1CEB00DA_u64);

        for backend in &backends {
            group.bench_with_input(BenchmarkId::new(backend.label(), count), &count, |b, &_count| {
                let backend = *backend;
                b.iter_custom(|iters| {
                    let mut total = Duration::ZERO;
                    let mut insert_index = 0usize;
                    for _ in 0..iters {
                        let (mut filter, path) = prepare_filter(backend, &preload, "insert_test")
                            .unwrap()
                            .into_parts();

                        let token = &insert_tokens[insert_index % insert_tokens.len()];
                        insert_index = insert_index.wrapping_add(1);
                        let start = Instant::now();
                        filter.insert(black_box(token)).unwrap();
                        black_box(filter.test(black_box(token)));
                        total += start.elapsed();

                        drop(filter);
                        cleanup_path(path);
                    }
                    total
                });
            });
        }
    }

    group.finish();
}

#[cfg(feature = "bench-insert-and-test")]
criterion_group!(benches, bench_insert_and_test);
#[cfg(not(feature = "bench-insert-and-test"))]
criterion_group!(benches, bench_insert, bench_test);
criterion_main!(benches);
