use std::time::Duration;

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use star_app::enclave::EnclaveRuntime;

const BENCH_GROUP_NAME: &str = "enclave_call_latency";
const OP_ONE_BENCH_NAME: &str = "op_one";

fn bench_enclave_call_latency(c: &mut Criterion) {
    let enclave = EnclaveRuntime::from_env().expect("enclave");
    assert_eq!(enclave.one().expect("OP_ONE"), [1; 32]);

    let mut group = c.benchmark_group(BENCH_GROUP_NAME);
    group.sample_size(40);
    group.measurement_time(Duration::from_secs(6));
    group.throughput(Throughput::Elements(1));

    group.bench_function(OP_ONE_BENCH_NAME, |b| {
        b.iter(|| {
            let response = enclave.one().expect("OP_ONE");
            black_box(response);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_enclave_call_latency);
criterion_main!(benches);
