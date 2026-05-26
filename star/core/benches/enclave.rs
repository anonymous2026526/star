use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rand::{rngs::OsRng, RngCore};
use star_core::client::User;
use star_core::secure_channel::client::SecureChannelClient;
use star_core::secure_channel::server::SecureChannelServer;
use star_core::serv::core::cred::Credential;
use star_core::serv::core::token::Token;
use star_core::serv::enclave::Enclave;

const PERIOD: u64 = 1;
const MAX_TOKENS_PER_CREDENTIAL: u32 = 3;
const ROUTE_BUCKETS: u32 = 1024;

struct BenchFixture {
    server: SecureChannelServer,
    aad: Vec<u8>,
    info: Vec<u8>,
    enc: Vec<u8>,
    ciphertext: Vec<u8>,
    credential: Credential,
    count: [u8; 8],
    period: [u8; 8],
    current_period: [u8; 8],
    max_tickets: [u8; 8],
    token_sk: [u8; 32],
    token_for_route: Token,
    token_payload: Vec<u8>,
    response_key: [u8; 32],
    nonce: [u8; 12],
}

fn build_fixture() -> BenchFixture {
    let mut rng = OsRng;

    let mut token_sk = [0u8; 32];
    let mut hpke_sk = [0u8; 32];
    let mut response_key = [0u8; 32];
    let mut nonce = [0u8; 12];
    rng.fill_bytes(&mut token_sk);
    rng.fill_bytes(&mut hpke_sk);
    rng.fill_bytes(&mut response_key);
    rng.fill_bytes(&mut nonce);

    let enclave = Enclave::new(MAX_TOKENS_PER_CREDENTIAL, token_sk, hpke_sk)
        .expect("failed to initialize enclave");
    let server =
        SecureChannelServer::new(hpke_sk).expect("failed to initialize secure channel server");

    let sender =
        SecureChannelClient::new(enclave.public_key()).expect("failed to initialize sender");
    let mut user = User::new(MAX_TOKENS_PER_CREDENTIAL as u64, sender);

    let (cipher, ck) = user.request_credential(&mut rng);
    let credential_bytes = enclave.issue_cred(&cipher, &mut rng);
    user.receive_credential(&credential_bytes, &ck);

    let request_plain = user.request_auth_plain(0, PERIOD);
    let envelope = user
        .request_auth(0, PERIOD, &mut rng)
        .expect("failed to build token request");

    const AAD_LEN: usize = 32;
    const INFO_LEN: usize = 32;
    const ENC_LEN: usize = 32;

    let aad = envelope[..AAD_LEN].to_vec();
    let info = envelope[AAD_LEN..AAD_LEN + INFO_LEN].to_vec();
    let enc = envelope[AAD_LEN + INFO_LEN..AAD_LEN + INFO_LEN + ENC_LEN].to_vec();
    let ciphertext = envelope[AAD_LEN + INFO_LEN + ENC_LEN..].to_vec();

    let credential = Credential::from_bytes(&request_plain[0..64]);
    let count: [u8; 8] = request_plain[64..72]
        .try_into()
        .expect("fixed-length slice");
    let period: [u8; 8] = request_plain[72..80]
        .try_into()
        .expect("fixed-length slice");
    let current_period = PERIOD.to_be_bytes();
    let max_tickets = (MAX_TOKENS_PER_CREDENTIAL as u64).to_be_bytes();

    let token_for_route = Token::random(credential.uid, period, count);
    let token_payload = token_for_route.to_bytes();

    BenchFixture {
        server,
        aad,
        info,
        enc,
        ciphertext,
        credential,
        count,
        period,
        current_period,
        max_tickets,
        token_sk,
        token_for_route,
        token_payload,
        response_key,
        nonce,
    }
}

fn bench_issue_token_stages(c: &mut Criterion) {
    let fixture = build_fixture();

    let mut group = c.benchmark_group("enclave.issue_token.stages");

    group.throughput(Throughput::Bytes(
        (fixture.aad.len() + fixture.info.len() + fixture.enc.len() + fixture.ciphertext.len())
            as u64,
    ));
    group.bench_function("decrypt", |b| {
        b.iter(|| {
            let plain = fixture
                .server
                .receive_hpke(
                    black_box(fixture.enc.as_slice()),
                    black_box(fixture.info.as_slice()),
                    black_box(fixture.aad.as_slice()),
                    black_box(fixture.ciphertext.as_slice()),
                )
                .expect("decryption failed");
            black_box(plain);
        });
    });

    group.throughput(Throughput::Bytes(80));
    group.bench_function("verify", |b| {
        b.iter(|| {
            let ok = fixture.credential.is_valid(
                black_box(fixture.period),
                black_box(fixture.count),
                black_box(fixture.current_period),
                black_box(fixture.max_tickets),
                black_box(fixture.token_sk),
            );
            black_box(ok);
        });
    });

    group.throughput(Throughput::Bytes(fixture.token_payload.len() as u64));
    group.bench_function("token_generation", |b| {
        b.iter(|| {
            let token = Token::random(
                black_box(fixture.credential.uid),
                black_box(fixture.period),
                black_box(fixture.count),
            );
            black_box(token.to_bytes());
        });
    });

    group.throughput(Throughput::Elements(1));
    group.bench_function("route", |b| {
        b.iter(|| {
            let route = fixture.token_for_route.route(black_box(ROUTE_BUCKETS));
            black_box(route);
        });
    });

    group.throughput(Throughput::Bytes(fixture.token_payload.len() as u64));
    group.bench_function("encrypt", |b| {
        b.iter(|| {
            let aead = ChaCha20Poly1305::new_from_slice(&fixture.response_key)
                .expect("response key must be 32 bytes");
            let nonce = Nonce::from_slice(&fixture.nonce);
            let ciphertext = aead
                .encrypt(nonce, black_box(fixture.token_payload.as_slice()))
                .expect("encryption failure");
            black_box(ciphertext);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_issue_token_stages);
criterion_main!(benches);
