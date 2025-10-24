use criterion::{criterion_group, criterion_main, Criterion};

fn trivial_bench(c: &mut Criterion) {
    c.bench_function("trivial", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, trivial_bench);
criterion_main!(benches);
