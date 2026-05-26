mod common;

use std::{
    env,
    sync::{Arc, Mutex as StdMutex},
    time::Duration,
};

use criterion::{BatchSize, Criterion, Throughput, black_box, criterion_group, criterion_main};
use rand_core::OsRng;
use simple_filter_box::{core::Entry, dashmap::DashMapFilterBox};
use star_app::{
    KeyManagerRuntime,
    enclave::{EnclaveRuntime, EnclaveRuntimes},
    filter::StarFilter,
    install_key_manager_keys_for_enclave, install_key_manager_keys_for_pool,
};
use star_core::{client::User, secure_channel::client::SecureChannelClient};
use tokio::{
    runtime::{Builder, Runtime},
    task::JoinSet,
};

type HashMapStarFilter = StarFilter<DashMapFilterBox>;
type SharedStarFilter = Arc<HashMapStarFilter>;

const MAX_COUNT: u32 = 1_000_000;

const FIXED_PREFIX_MAX: u32 = 2;
const FIXED_CONCURRENCY: usize = FIXED_PREFIX_MAX as usize;
const DEFAULT_FILTER_ENCLAVES: u8 = 2;

const ISSUE_TOKEN_OPS_PER_WORKER: usize = 64;
const ISSUE_TOKEN_BATCH_OPS: usize = FIXED_CONCURRENCY * ISSUE_TOKEN_OPS_PER_WORKER;

const CHECK_DUPLICATION_OPS_PER_WORKER: usize = 4096;
const CHECK_DUPLICATION_BATCH_OPS: usize =
    FIXED_CONCURRENCY * CHECK_DUPLICATION_OPS_PER_WORKER;

const ISSUE_TOKEN_GROUP_NAME: &str = "star_filter_issue_token_and_route";
const CHECK_DUPLICATION_GROUP_NAME: &str = "star_filter_check_duplication";
const DIAGNOSTICS_GROUP_NAME: &str = "star_filter_diagnostics";

const LATENCY_BENCH_NAME: &str = "latency";
const PARALLEL_THROUGHPUT_BENCH_NAME: &str = "parallel_throughput";

fn current_thread_runtime() -> Runtime {
    Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime")
}

fn multi_thread_runtime(worker_threads: usize) -> Runtime {
    Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .build()
        .expect("tokio runtime")
}

fn fixed_prefixes() -> Vec<u32> {
    (0..FIXED_PREFIX_MAX).collect()
}

fn filter_enclave_count() -> u8 {
    env::var("STAR_FILTER_BENCH_ENCLAVES")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_FILTER_ENCLAVES)
}

fn build_filter() -> HashMapStarFilter {
    let filter_box = DashMapFilterBox::new();
    let enclave = EnclaveRuntimes::from_env(filter_enclave_count()).expect("filter enclave pool");

    StarFilter::new(filter_box, fixed_prefixes(), FIXED_PREFIX_MAX, enclave)
}

struct TokenRequestGenerator {
    issue_enclave: EnclaveRuntime,
    user: User,
    counter: u64,
    period: u64,
    rng: OsRng,
}

impl TokenRequestGenerator {
    fn new(issue_enclave: EnclaveRuntime) -> Self {
        let mut rng = OsRng;
        let user = register_user(&issue_enclave, &mut rng);

        Self {
            issue_enclave,
            user,
            counter: 0,
            period: u64::from_be_bytes(constant_time_utils::time::current_period()),
            rng,
        }
    }

    fn next_request(&mut self) -> Vec<u8> {
        let current_period = u64::from_be_bytes(constant_time_utils::time::current_period());
        if current_period != self.period || self.counter >= MAX_COUNT as u64 {
            self.period = current_period;
            self.user = register_user(&self.issue_enclave, &mut self.rng);
            self.counter = 0;
        }

        let request = self
            .user
            .request_auth(self.counter, self.period, &mut self.rng)
            .expect("token request");

        self.counter += 1;
        request
    }
}

fn build_keyed_filter_and_request_generator() -> (HashMapStarFilter, TokenRequestGenerator) {
    let _aesm_proxy = common::aesm_proxy::AesmProxyGuard::start();

    let key_manager = KeyManagerRuntime::from_env_with_sealed_keys(&[]).expect("key manager");
    let filter_enclave =
        EnclaveRuntimes::from_env(filter_enclave_count()).expect("filter enclave pool");
    let issue_enclave = EnclaveRuntime::from_env().expect("issue enclave");

    install_key_manager_keys_for_pool(&key_manager, &filter_enclave)
        .expect("install filter enclave benchmark keys");
    install_key_manager_keys_for_enclave(&key_manager, &issue_enclave)
        .expect("install issue enclave benchmark keys");

    let filter = StarFilter::new(
        DashMapFilterBox::new(),
        fixed_prefixes(),
        FIXED_PREFIX_MAX,
        filter_enclave,
    );
    let request_generator = TokenRequestGenerator::new(issue_enclave);

    (filter, request_generator)
}

fn build_request(enclave: &EnclaveRuntime) -> Vec<u8> {
    let mut rng = OsRng;
    let mut user = register_user(enclave, &mut rng);
    let period = u64::from_be_bytes(constant_time_utils::time::current_period());

    user.request_auth(0, period, &mut rng)
        .expect("token request")
}

fn register_user(enclave: &EnclaveRuntime, rng: &mut OsRng) -> User {
    let sender =
        SecureChannelClient::new(enclave.public_key().expect("public key")).expect("sender init");
    let mut user = User::new(MAX_COUNT as u64, sender);

    let (cred_request, shared_key) = user.request_credential(rng);
    let cred_cipher = enclave.issue_cred(&cred_request).expect("issue credential");
    user.receive_credential(&cred_cipher, &shared_key);

    user
}

fn route_for_token_index(idx: u64) -> u32 {
    (idx as u32) % FIXED_PREFIX_MAX
}

fn advance_batch(counter: &mut u64, batch_ops: u64) -> u64 {
    let start = *counter;
    *counter = (*counter).wrapping_add(batch_ops);
    start
}

fn token_from_counter(counter: u64) -> Entry {
    let mut token = [0u8; 32];
    token[0..8].copy_from_slice(&counter.to_be_bytes());
    token
}

fn next_request_for_measurement(
    request_generator: &Arc<StdMutex<TokenRequestGenerator>>,
) -> Vec<u8> {
    request_generator
        .lock()
        .expect("request generator mutex")
        .next_request()
}

fn next_request_batch_for_measurement(
    request_generator: &Arc<StdMutex<TokenRequestGenerator>>,
    batch_ops: usize,
) -> Vec<Vec<u8>> {
    let mut request_generator = request_generator.lock().expect("request generator mutex");

    (0..batch_ops)
        .map(|_| request_generator.next_request())
        .collect()
}

async fn run_issue_token_and_route_parallel_batch(
    filter: SharedStarFilter,
    requests: Vec<Vec<u8>>,
) {
    let mut worker_requests = Vec::with_capacity(FIXED_CONCURRENCY);
    for _ in 0..FIXED_CONCURRENCY {
        worker_requests.push(Vec::with_capacity(ISSUE_TOKEN_OPS_PER_WORKER));
    }

    for (idx, request) in requests.into_iter().enumerate() {
        worker_requests[idx % FIXED_CONCURRENCY].push(request);
    }

    let mut joins = JoinSet::new();

    for requests in worker_requests {
        let filter = Arc::clone(&filter);

        joins.spawn(async move {
            for request in requests {
                let result = filter
                    .issue_token_and_route(black_box(request.as_slice()))
                    .await
                    .expect("issue token and route");
                black_box(result);
            }
        });
    }

    while let Some(joined) = joins.join_next().await {
        joined.expect("join issue token worker");
    }
}

fn bench_issue_token_and_route(c: &mut Criterion) {
    let latency_rt = current_thread_runtime();
    let throughput_rt = multi_thread_runtime(FIXED_CONCURRENCY);

    let (filter, request_generator) = build_keyed_filter_and_request_generator();
    let filter = Arc::new(filter);
    let request_generator = Arc::new(StdMutex::new(request_generator));

    let mut group = c.benchmark_group(ISSUE_TOKEN_GROUP_NAME);

    group.throughput(Throughput::Elements(1));
    group.bench_function(LATENCY_BENCH_NAME, |b| {
        let filter = Arc::clone(&filter);
        let request_generator = Arc::clone(&request_generator);

        b.to_async(&latency_rt).iter_batched(
            || next_request_for_measurement(&request_generator),
            |request| {
                let filter = Arc::clone(&filter);

                async move {
                    let result = filter
                        .issue_token_and_route(black_box(request.as_slice()))
                        .await
                        .expect("issue token and route");
                    black_box(result);
                }
            },
            BatchSize::PerIteration,
        )
    });

    group.throughput(Throughput::Elements(ISSUE_TOKEN_BATCH_OPS as u64));
    group.bench_function(PARALLEL_THROUGHPUT_BENCH_NAME, |b| {
        let filter = Arc::clone(&filter);
        let request_generator = Arc::clone(&request_generator);

        b.to_async(&throughput_rt).iter_batched(
            || next_request_batch_for_measurement(&request_generator, ISSUE_TOKEN_BATCH_OPS),
            |requests| {
                let filter = Arc::clone(&filter);

                async move {
                    run_issue_token_and_route_parallel_batch(filter, requests).await;
                }
            },
            BatchSize::PerIteration,
        )
    });

    group.finish();
}

async fn run_check_duplication_parallel_batch(filter: SharedStarFilter, batch_start: u64) {
    let mut joins = JoinSet::new();

    for worker in 0..FIXED_CONCURRENCY {
        let filter = Arc::clone(&filter);

        joins.spawn(async move {
            for offset in 0..CHECK_DUPLICATION_OPS_PER_WORKER {
                let local_index = offset * FIXED_CONCURRENCY + worker;
                let idx = batch_start + local_index as u64;

                let token = token_from_counter(idx);
                let route = route_for_token_index(idx);
                let result = filter.check_duplication(black_box(&token), black_box(route));
                black_box(result);
            }
        });
    }

    while let Some(joined) = joins.join_next().await {
        joined.expect("join check_duplication worker");
    }
}

fn bench_check_duplication(c: &mut Criterion) {
    let throughput_rt = multi_thread_runtime(FIXED_CONCURRENCY);

    let latency_filter = Arc::new(build_filter());
    let throughput_filter = Arc::new(build_filter());

    let mut latency_counter = 0u64;
    let mut throughput_batch_start = 0u64;
    let throughput_batch_ops = CHECK_DUPLICATION_BATCH_OPS as u64;

    let mut group = c.benchmark_group(CHECK_DUPLICATION_GROUP_NAME);
    group.sample_size(40);
    group.measurement_time(Duration::from_secs(4));

    group.throughput(Throughput::Elements(1));
    group.bench_function(LATENCY_BENCH_NAME, |b| {
        let filter = Arc::clone(&latency_filter);

        b.iter_batched(
            || {
                let idx = advance_batch(&mut latency_counter, 1);
                let token = token_from_counter(idx);
                let route = route_for_token_index(idx);
                (token, route)
            },
            |(token, route)| {
                let result = filter.check_duplication(black_box(&token), black_box(route));
                black_box(result);
            },
            BatchSize::PerIteration,
        )
    });

    group.throughput(Throughput::Elements(throughput_batch_ops));
    group.bench_function(PARALLEL_THROUGHPUT_BENCH_NAME, |b| {
        let filter = Arc::clone(&throughput_filter);

        b.to_async(&throughput_rt).iter(|| {
            let filter = Arc::clone(&filter);
            let start = advance_batch(&mut throughput_batch_start, throughput_batch_ops);

            async move {
                run_check_duplication_parallel_batch(filter, start).await;
            }
        });
    });

    group.finish();
}

fn bench_diagnostics(c: &mut Criterion) {
    if env::var_os("STAR_FILTER_BENCH_DIAGNOSTICS").is_none() {
        return;
    }

    let _aesm_proxy = common::aesm_proxy::AesmProxyGuard::start();

    let rt = current_thread_runtime();
    let key_manager = KeyManagerRuntime::from_env_with_sealed_keys(&[]).expect("key manager");

    let request_enclave = EnclaveRuntime::from_env().expect("request enclave");
    install_key_manager_keys_for_enclave(&key_manager, &request_enclave)
        .expect("install request enclave benchmark keys");
    let request = Arc::new(build_request(&request_enclave));

    let mut group = c.benchmark_group(DIAGNOSTICS_GROUP_NAME);
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(1));

    let public_key_enclave = EnclaveRuntime::from_env().expect("public key enclave");
    install_key_manager_keys_for_enclave(&key_manager, &public_key_enclave)
        .expect("install public key enclave benchmark keys");
    let public_key_enclave = Arc::new(StdMutex::new(public_key_enclave));

    group.throughput(Throughput::Elements(1));
    group.bench_function("public_key_ecall", |b| {
        let enclave = Arc::clone(&public_key_enclave);

        b.iter(|| {
            let key = enclave
                .lock()
                .expect("enclave mutex")
                .public_key()
                .expect("public key");
            black_box(key);
        });
    });

    let issue_token_enclave = EnclaveRuntime::from_env().expect("issue token enclave");
    install_key_manager_keys_for_enclave(&key_manager, &issue_token_enclave)
        .expect("install issue token enclave benchmark keys");
    let issue_token_enclave = Arc::new(StdMutex::new(issue_token_enclave));

    group.throughput(Throughput::Elements(1));
    group.bench_function("issue_token_direct_ecall", |b| {
        let enclave = Arc::clone(&issue_token_enclave);
        let request = Arc::clone(&request);

        b.iter(|| {
            let token = enclave
                .lock()
                .expect("enclave mutex")
                .issue_token(
                    black_box(request.as_slice()),
                    constant_time_utils::time::current_period(),
                )
                .expect("issue token");
            black_box(token);
        });
    });

    group.throughput(Throughput::Elements(1));
    group.bench_function("spawn_blocking_noop", |b| {
        b.to_async(&rt).iter(|| async {
            let value = tokio::task::spawn_blocking(|| 1u64)
                .await
                .expect("spawn_blocking join");
            black_box(value);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_issue_token_and_route,
    bench_check_duplication,
    bench_diagnostics
);

criterion_main!(benches);
