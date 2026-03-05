use crate::{
    blackjack::{Action, game::Game},
    player::Player,
};

pub struct Ai;

impl Player for Ai {
    fn ask_user(&mut self, game: &Game, legal_moves: &[Action]) -> Action {
        todo!()
    }
}
