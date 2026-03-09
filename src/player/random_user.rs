use crate::{
    blackjack::{Action, game::Game},
    player::Player,
};
use rand::seq::IteratorRandom;

#[derive(Default)]
pub struct RandomUser {}

impl Player for RandomUser {
    fn ask_user(&mut self, _game: &Game, legal_moves: &[Action]) -> Action {
        *legal_moves
            .iter()
            .filter(|x| **x != Action::Insurance) //TODO: remove once Insurance is implemented
            .choose(&mut rand::rng())
            .unwrap()
    }
}
