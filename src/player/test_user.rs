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
    fn ask_user(&mut self, _game: &Game, _legal_movess: &[Action]) -> Action {
        self.index += 1;
        self.actions_to_do[self.index - 1]
    }
}
