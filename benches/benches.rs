extern crate criterion;
extern crate diff;

use criterion::Criterion;

criterion::criterion_group!(benches, bench_slice);
criterion::criterion_main!(benches);

fn bench_slice(c: &mut Criterion) {
    c.bench_function("empty", |b| {
        let slice = [0u8; 0];
        b.iter(|| ::diff::slice(&slice, &slice));
    });

    c.bench_function("10 equal items", |b| {
        let slice = [0u8; 10];
        b.iter(|| ::diff::slice(&slice, &slice));
    });

    c.bench_function("10 non-equal items", |b| {
        let (left, right) = ([0u8; 10], [1u8; 10]);
        b.iter(|| ::diff::slice(&left, &right));
    });

    c.bench_function("100 equal items", |b| {
        let slice = [0u8; 100];
        b.iter(|| ::diff::slice(&slice, &slice));
    });

    c.bench_function("100 non-equal items", |b| {
        let (left, right) = ([0u8; 100], [1u8; 100]);
        b.iter(|| ::diff::slice(&left, &right));
    });

    c.bench_function("1000 equal items", |b| {
        let slice = [0u8; 1000];
        b.iter(|| ::diff::slice(&slice, &slice));
    });

    c.bench_function("1000 non-equal items", |b| {
        let (left, right) = ([0u8; 1000], [1u8; 1000]);
        b.iter(|| ::diff::slice(&left, &right));
    });
}
