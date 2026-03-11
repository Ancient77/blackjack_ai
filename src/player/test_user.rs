use crate::{
    blackjack::{Action, game::Game},
    player::Player,
};

#[derive(Default)]
pub struct TestUser {
    actions_to_do: Vec<Action>,
    index: usize,
}

impl TestUser {
    pub fn new(actions_to_do: Vec<Action>) -> Self {
        TestUser {
            actions_to_do,
            index: 0,
        }
    }
}

impl Player for TestUser {
    fn ask_user(&mut self, _game: &Game, legal_moves: &[Action]) -> Action {
        let result = self.actions_to_do[self.index];

        if !legal_moves.contains(&result) {
            panic!("Player wanted to choose {:?}, but it was not offered", result)
        }

        self.index += 1;

        result
    }
}
