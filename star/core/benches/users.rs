use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rand::{rngs::OsRng, RngCore};
use star_core::client::User;
use star_core::secure_channel::client::SecureChannelClient;
use star_core::serv::enclave::Enclave;

const PERIOD: u64 = 1;
const MAX_TOKENS_PER_CREDENTIAL: u32 = 3;
const ROUTE_BUCKETS: u32 = 1024;

struct UserBenchFixture {
    user: User,
    request_plain: Vec<u8>,
    request_plain_len: usize,
    request_envelope_len: usize,
}

fn build_user_fixture() -> UserBenchFixture {
    let mut rng = OsRng;

    let mut token_sk = [0u8; 32];
    let mut hpke_sk = [0u8; 32];
    rng.fill_bytes(&mut token_sk);
    rng.fill_bytes(&mut hpke_sk);

    let enclave = Enclave::new(MAX_TOKENS_PER_CREDENTIAL, token_sk, hpke_sk)
        .expect("failed to initialize enclave");
    let sender =
        SecureChannelClient::new(enclave.public_key()).expect("failed to initialize sender");
    let mut user = User::new(MAX_TOKENS_PER_CREDENTIAL as u64, sender);

    let (credential_request_envelope, credential_shared_key) = user.request_credential(&mut rng);
    let credential_cipher = enclave.issue_cred(&credential_request_envelope, &mut rng);
    user.receive_credential(&credential_cipher, &credential_shared_key);

    let request_plain = user.request_auth_plain(0, PERIOD);
    let request_plain_len = request_plain.len();
    let request_envelope_len = user
        .client
        .send_hpke(&request_plain, &mut rng)
        .expect("failed to encrypt auth request")
        .len();

    UserBenchFixture {
        user,
        request_plain,
        request_plain_len,
        request_envelope_len,
    }
}

fn bench_user_latency(c: &mut Criterion) {
    let fixture = build_user_fixture();

    let mut group = c.benchmark_group("user.latency");

    group.throughput(Throughput::Bytes(fixture.request_plain_len as u64));
    group.bench_function("request_auth_plain", |b| {
        let UserBenchFixture { mut user, .. } = build_user_fixture();
        b.iter(|| {
            let request = user.request_auth_plain(black_box(0), black_box(PERIOD));
            black_box(request);
        });
    });

    group.throughput(Throughput::Bytes(fixture.request_envelope_len as u64));
    group.bench_function("encrypt", |b| {
        let UserBenchFixture {
            user,
            request_plain,
            ..
        } = build_user_fixture();
        let mut rng = OsRng;
        b.iter(|| {
            let envelope = user
                .client
                .send_hpke(black_box(request_plain.as_slice()), &mut rng)
                .expect("failed to encrypt auth request");
            black_box(envelope);
        });
    });

    group.throughput(Throughput::Elements(1));
    group.bench_function("route", |b| {
        let UserBenchFixture { mut user, .. } = build_user_fixture();
        b.iter(|| {
            let route = user.route(black_box(0), black_box(PERIOD), black_box(ROUTE_BUCKETS));
            black_box(route);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_user_latency);
criterion_main!(benches);
