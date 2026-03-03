use rand::{Rng, RngExt, SeedableRng, rngs::{StdRng, ThreadRng}};

use crate::{
    ai::ask_ai,
    blackjack::{Action, card::Card, game_config::GameConfig, hand::Hand},
};

#[derive(Clone)]
enum GameResult {
    Bust,
    DealerBust,
    Win,
    DealerWin,
    Tie,
    Surrender,
    Split,
}

impl GameResult {
    pub fn score(&self) -> f32 {
        match self {
            GameResult::Bust | GameResult::DealerWin => -1.0,
            GameResult::Tie => 0.0,
            GameResult::Surrender => -0.5,
            GameResult::Win | GameResult::DealerBust => 1.0,
            GameResult::Split => unreachable!("split is resolved in play()"),
        }
    }
}

#[derive(Clone)]
pub struct Game<R: Rng> {
    rng: R,
    dealer_hand: Hand,
    player_hand: Hand,
    result: Option<GameResult>,
    double_down: bool,
    config: GameConfig,
}

impl Default for Game<ThreadRng> {
    fn default() -> Self {
        let mut rng = rand::rng();
        Self {
            dealer_hand: Hand {
                cards: vec![rng.random()],
            },
            player_hand: Hand {
                cards: vec![rng.random(), rng.random()],
            },
            result: None,
            double_down: false,
            config: GameConfig::default(),
            rng: rng,
        }
    }
}

impl Game<ThreadRng>  {
        pub fn new() -> Self {
        Self::default()
    }
}

impl Game<StdRng>  {
    pub fn new_seeded(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let dealer_hand = Hand { cards: vec![rng.random()] };
        let player_hand = Hand { cards: vec![rng.random(), rng.random()] };

        Self {
            rng,
            dealer_hand,
            player_hand,
            result: None,
            double_down: false,
            config: GameConfig::default(),
        }
    }
}

impl<R: Rng> Game<R> {
    
    pub fn from_split(card: Card, dealer_hand: Hand, config: GameConfig, mut rng: R) -> Self{
         Self {
            dealer_hand,
            player_hand: Hand {
                cards: vec![card, rng.random()],
            },
            result: None,
            double_down: false,
            config: config,
            rng
    }
        
    }
    
    fn draw_card(&mut self) -> Card {
        self.rng.random()
    }

    pub fn game_loop(mut self) -> f32 {
        self.ai_loop();

        if let Some(GameResult::Split) = &self.result {
            let cards = self.player_hand.cards;
            let score_a = Game::from_split(cards[0], self.dealer_hand.clone(), self.config, self.rng)
                .game_loop();
            let score_b = Game::from_split(cards[1], self.dealer_hand, self.config, self.rng)
                .game_loop();
            return score_a + score_b;
        }

        self.dealer_loop();

        self.calculate_outcome()
    }

    fn ai_loop(&mut self) {
        let mut legal_moves: Vec<Action> = Vec::with_capacity(std::mem::variant_count::<Action>());
        // Blackjack
        //TODO: can only be tied by other blackjack
        if self.player_hand.is_natural_blackjack() {
            return;
        }
        if self.dealer_hand.cards.contains(&Card::Ace) {
            // Ace
            legal_moves.push(Action::Insurance);
        }

        legal_moves.append(&mut Vec::from([
            Action::Hit,
            Action::Stand,
            Action::DoubleDown,
            Action::Split,
            Action::Surrender,
        ]));

        loop {
            match ask_ai(self, &legal_moves) {
                Action::Hit => self.player_hand.cards.push(self.draw_card()),
                Action::Stand => return,
                Action::DoubleDown => {
                    self.double_down = true;
                    self.player_hand.cards.push(self.draw_card());
                    if self.player_hand.is_bust() {
                        self.result = Some(GameResult::Bust);
                    }
                    return;
                }
                Action::Split => {
                    self.result = Some(GameResult::Split);
                    return;
                }
                Action::Surrender => {
                    self.result = Some(GameResult::Surrender);
                    return;
                }
                Action::Insurance => todo!(),
            }

            if self.player_hand.calc_points_ace_as_one() == 21 {
                return;
            }

            if self.player_hand.calc_points_ace_as_one() > 21 {
                self.result = Some(GameResult::Bust);
                return;
            }
        }
    }

    fn dealer_loop(&mut self) {
        if self.result.is_some() {
            return;
        }

        //Hit until 17
        while self.dealer_hand.calc_points_best_possible() < 17 {
            let new_card = self.draw_card();
            self.dealer_hand.cards.push(new_card);
        }

        if self.dealer_hand.is_bust() {
            self.result = Some(GameResult::DealerBust);
        } else if self.dealer_hand.calc_points_best_possible()
            == self.player_hand.calc_points_best_possible()
        {
            if self.player_hand.is_natural_blackjack() && !self.dealer_hand.is_natural_blackjack() {
                self.result = Some(GameResult::Win);
            } else if !self.player_hand.is_natural_blackjack() && self.dealer_hand.is_natural_blackjack() {
                //This could be checked earlier
                self.result = Some(GameResult::DealerWin);
            } else {
                self.result = Some(GameResult::Tie);
            }
        } else if self.dealer_hand.calc_points_best_possible()
            < self.player_hand.calc_points_best_possible()
        {
            self.result = Some(GameResult::Win);
        } else {
            self.result = Some(GameResult::DealerWin)
        }
    }

    fn calculate_outcome(&self) -> f32 {
        self
            .result.as_ref()
            .expect("Why dafaq is game not done at the end?")
            .score()
            * if self.double_down { 2.0 } else { 1.0 }
    }
}
