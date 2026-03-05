use crate::{
    blackjack::{Action, game::Game},
    player::Player,
};
use std::io;
pub struct CommandLineUser;

impl Player for CommandLineUser {
    fn ask_user(&mut self, game: &Game, legal_moves: &[Action]) -> Action {
        println!("Dealer Hand: {:?}", game.dealer_hand);
        println!("Your Hand: {:?}", game.player_hand);

        println!("Legal Actions: {:?}", legal_moves);

        let mut input = String::new();

        // 4. Read from standard input into the string
        io::stdin().read_line(&mut input).expect("Failed to read line");

        legal_moves[input.trim().parse::<usize>().expect("Couldnt Parse String")]
    }
}
