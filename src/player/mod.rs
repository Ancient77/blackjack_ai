pub mod ai;
pub mod command_line_user;
pub mod random_user;
#[cfg(test)]
pub mod test_user;

use crate::blackjack::{Action, game::Game};

pub trait Player {
    fn ask_user(&mut self, game: &Game, legal_moves: &[Action]) -> Action;
}
