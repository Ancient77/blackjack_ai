#![feature(variant_count)]

use crate::blackjack::game::Game;

mod ai;
mod blackjack;

fn main() {
    let score = Game::new().game_loop();
}
