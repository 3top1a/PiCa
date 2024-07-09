use chess::Board;
use criterion::{criterion_group, criterion_main, Criterion};
use pica::{engine::Engine, eval::eval, time::TimeManager, utils::History};

pub fn criterion_benchmark(c: &mut Criterion) {
    // c.bench_function("eval", |b| b.iter(|| eval(&Board::default())));
    c.bench_function("search d5", |b| b.iter(|| {
        let mut e = Engine::new(64);
        e.start(Board::default(), TimeManager { max_depth: Some(5), max_nodes: None, board_time: None, max_allowed_time_now: None }, History::new());
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
