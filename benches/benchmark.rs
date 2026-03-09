use blackjack_ai::{blackjack::game::Game, player::random_user::RandomUser};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("blackjack", |b| {
        b.iter(|| black_box(Game::new(RandomUser {}).game_loop()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
