use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("maybe_field_nastran_float", |b| {
        b.iter(|| nastran::datfile::maybe_field(black_box(b"11.22+7")))
    });
    c.bench_function("maybe_field_float", |b| {
        b.iter(|| nastran::datfile::maybe_field(black_box(b"11.22e+7")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
