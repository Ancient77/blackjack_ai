#![feature(variant_count)]

use crate::blackjack::game_loop;

mod ai;
mod blackjack;

fn main() {
    game_loop(None);
}
