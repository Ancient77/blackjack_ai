#![feature(variant_count)]

use crate::{blackjack::game::Game, player::command_line_user::CommandLineUser};

mod blackjack;
mod player;

fn main() {
    let score = Game::new(CommandLineUser).game_loop();
}
