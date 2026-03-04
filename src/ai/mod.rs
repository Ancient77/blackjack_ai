use std::io;

use crate::blackjack;

pub fn ask_ai(game: &blackjack::game::Game, legal_moves: &[blackjack::Action]) -> blackjack::Action {
    //TODO: Currently ask console
    println!("Dealer Hand: {:?}", game.dealer_hand);
    println!("Your Hand: {:?}", game.player_hand);

    println!("Legal Actions: {:?}", legal_moves);

    let mut input = String::new();

    // 4. Read from standard input into the string
    io::stdin().read_line(&mut input).expect("Failed to read line");

    legal_moves[input.trim().parse::<usize>().expect("Couldnt Parse String")].clone()
}
