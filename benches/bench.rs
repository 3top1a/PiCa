use chess::Board;
use criterion::{criterion_group, criterion_main, Criterion};
use pica::eval::eval;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("eval", |b| b.iter(|| eval(&Board::default())));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
