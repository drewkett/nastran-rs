use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(_: &mut Criterion) {
    // c.bench_function("maybe_field_nastran_float", |b| {
    //     b.iter(|| nastran::bdf::v0::parser::maybe_field(black_box(b"11.22+7")))
    // });
    // c.bench_function("maybe_field_float", |b| {
    //     b.iter(|| nastran::bdf::v0::parser::maybe_field(black_box(b"11.22e+7")))
    // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
