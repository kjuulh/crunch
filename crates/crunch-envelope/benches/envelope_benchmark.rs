use criterion::{criterion_group, criterion_main, Criterion};
use crunch_envelope::{unwrap, wrap};

fn envelope_capnp_benchmark(content: &[u8]) {
    let out = wrap("some-domain", "some-entity", content);

    let _ = unwrap(&out).expect("to be able to unwrap capnp message");
}

fn envelope_json_benchmark(content: &[u8]) {
    let out = crunch_envelope::json::wrap("some-domain", "some-entity", content);

    let _ = crunch_envelope::json::unwrap(&out).expect("to be able to unwrap capnp message");
}

fn envelope_proto_benchmark(content: &[u8]) {
    let out = crunch_envelope::proto::wrap("some-domain", "some-entity", content);

    let _ = crunch_envelope::proto::unwrap(&out).expect("to be able to unwrap capnp message");
}

fn criterion_benchmark(c: &mut Criterion) {
    let large_content: [u8; 10000] = [0; 10000];

    c.bench_function("envelope::capnp", |b| {
        b.iter(|| envelope_capnp_benchmark(&large_content))
    });
    c.bench_function("envelope::json", |b| {
        b.iter(|| envelope_json_benchmark(&large_content))
    });
    c.bench_function("envelope::proto", |b| {
        b.iter(|| envelope_proto_benchmark(&large_content))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
