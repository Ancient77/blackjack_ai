use crate::ai::ask_ai;
use rand::RngExt;

// Difference between mutlicard 21 and natural 21 is not implemented
// Soft17 not implemented
// Even money not implemented
// Blackjack 3:2 payout (1.5) is not implemented
// Aces can be 11 or 1
// Split is not implemented
// Deck is infinite
struct GameVariations {
    action_on_17: ActionOn17,
    peeking: PeekingNaturalBlackjack,
}

//TODO: Explenation
enum ActionOn17 {
    Hard17,
    Soft17,
}

enum PeekingNaturalBlackjack {
    //If dealer gets Ace or 10, they reveal other card early
    HoleCardGame,
    //dealer doesn't peek second card
    NoHoleCardGame,
}

pub enum Actions {
    Hit,
    Stand,
    DoubleDown,
    Split,
    Surrender,
    Insurance,
}

enum GameResult {
    Bust,
    DealerBust,
    Win,
    DealerWin,
    Tie,
    Surrender,
    Split(f32),
}

impl GameResult {
    pub fn score(&self) -> f32 {
        match self {
            GameResult::Bust | GameResult::DealerWin => -1.0,
            GameResult::Tie => 0.0,
            GameResult::Surrender => -0.5,
            GameResult::Win | GameResult::DealerBust => 1.0,
            GameResult::Split(score) => *score,
        }
    }
}

pub struct GameState {
    dealer_hand: Hand,
    player_hand: Hand,
    result: Option<GameResult>,
    double_down: bool,
}

struct Hand {
    cards: Vec<Card>,
}

impl Hand {
    pub fn is_natural_blackjack(&self) -> bool {
        self.cards.len() == 2 && self.cards.contains(&Card::Ace) && self.cards.contains(&Card::Ten)
    }

    pub fn is_bust(&self) -> bool {
        self.calc_points_ace_as_one() > 21
    }

    pub fn calc_points_best_possible(&self) -> i32 {
        let mut result = self.calc_points_ace_as_eleven();
        if result <= 21 {
            return result;
        }

        for _ in 0..self.cards.iter().filter(|card| **card == Card::Ace).count() {
            result -= 10;
            if result <= 21 {
                return result;
            }
        }

        result
    }

    //TODO: Multiple aces can be 1 and 11 at the same time
    pub fn calc_points_ace_as_one(&self) -> i32 {
        self.cards.iter().map(|card| card.card_to_score_ace_as_var(1)).sum()
    }

    pub fn calc_points_ace_as_eleven(&self) -> i32 {
        self.cards.iter().map(|card| card.card_to_score_ace_as_var(11)).sum()
    }
}

#[derive(PartialEq, Eq)]
enum Card {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Card {
    pub fn card_to_score_ace_as_var(&self, ace_as: i32) -> i32 {
        match self {
            Card::Two => 2,
            Card::Three => 3,
            Card::Four => 4,
            Card::Five => 5,
            Card::Six => 6,
            Card::Seven => 7,
            Card::Eight => 8,
            Card::Nine => 9,
            Card::Ten | Card::Jack | Card::Queen | Card::King => 10,
            Card::Ace => ace_as,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            dealer_hand: Hand {
                cards: vec![new_random_card()],
            },
            player_hand: Hand {
                cards: vec![new_random_card(), new_random_card()],
            },
            result: None,
            double_down: false,
        }
    }
}

pub fn game_loop(pre_game_state: Option<GameState>) -> f32 {
    let mut game_state = pre_game_state.unwrap_or_default();

    ai_loop(&mut game_state);

    // if game_state.result == Some(GameResult::Split(0.0)){
    //     let first_game
    // }

    dealer_loop(&mut game_state);

    calculate_outcome(game_state)
}

fn ai_loop(game_state: &mut GameState) {
    let mut legal_moves: Vec<Actions> = Vec::with_capacity(std::mem::variant_count::<Actions>());
    // Blackjack
    //TODO: can only be tied by other blackjack
    if game_state.player_hand.is_natural_blackjack() {
        return;
    }
    if game_state.dealer_hand.cards.contains(&Card::Ace) {
        // Ace
        legal_moves.push(Actions::Insurance);
    }

    legal_moves.append(&mut Vec::from([
        Actions::Hit,
        Actions::Stand,
        Actions::DoubleDown,
        Actions::Split,
        Actions::Surrender,
    ]));

    loop {
        match ask_ai(game_state, &legal_moves) {
            Actions::Hit => game_state.player_hand.cards.push(new_random_card()),
            Actions::Stand => return,
            Actions::DoubleDown => {
                game_state.double_down = true;
                game_state.player_hand.cards.push(new_random_card());
                if game_state.player_hand.is_bust() {
                    game_state.result = Some(GameResult::Bust);
                }
                return;
            }
            Actions::Split => {
                game_state.result = Some(GameResult::Split(0.0));
                return;
            }
            Actions::Surrender => {
                game_state.result = Some(GameResult::Surrender);
                return;
            }
            Actions::Insurance => todo!(),
        }

        if game_state.player_hand.calc_points_ace_as_one() == 21 {
            return;
        }

        if game_state.player_hand.calc_points_ace_as_one() > 21 {
            game_state.result = Some(GameResult::Bust);
            return;
        }
    }
}

fn dealer_loop(game_state: &mut GameState) {
    if game_state.result.is_some() {
        return;
    }

    //Hit until 17
    while game_state.dealer_hand.calc_points_best_possible() < 17 {
        game_state.dealer_hand.cards.push(new_random_card());
    }

    if game_state.dealer_hand.is_bust() {
        game_state.result = Some(GameResult::DealerBust);
    } else if game_state.dealer_hand.calc_points_best_possible() == game_state.player_hand.calc_points_best_possible() {
        if game_state.player_hand.is_natural_blackjack() && !game_state.dealer_hand.is_natural_blackjack() {
            game_state.result = Some(GameResult::Win);
        } else if !game_state.player_hand.is_natural_blackjack() && game_state.dealer_hand.is_natural_blackjack() {
            //This could be checked earlier
            game_state.result = Some(GameResult::DealerWin);
        } else {
            game_state.result = Some(GameResult::Tie);
        }
    } else if game_state.dealer_hand.calc_points_best_possible() < game_state.player_hand.calc_points_best_possible() {
        game_state.result = Some(GameResult::Win);
    } else {
        game_state.result = Some(GameResult::DealerWin)
    }
}

fn new_random_card() -> Card {
    let mut rng = rand::rng();
    let random_number: u32 = rng.random_range(0..std::mem::variant_count::<Card>() as u32);
    match random_number {
        0 => Card::Ace,
        1 => Card::Two,
        2 => Card::Three,
        3 => Card::Four,
        4 => Card::Five,
        5 => Card::Six,
        6 => Card::Seven,
        7 => Card::Eight,
        8 => Card::Nine,
        9 => Card::Ten,
        10 => Card::Jack,
        11 => Card::Queen,
        12 => Card::King,
        _ => panic!("random_number can't be converted to Card"),
    }
}

fn calculate_outcome(game_state: GameState) -> f32 {
    game_state
        .result
        .expect("Why dafaq is game not done at the end?")
        .score()
        * if game_state.double_down { 2.0 } else { 1.0 }
}
