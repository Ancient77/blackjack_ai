use std::{cell::RefCell, rc::Rc};

use crate::{
    ai::ask_ai,
    blackjack::{
        Action,
        card::Card,
        card_source::{CardSource, RandomDeck},
        game_config::GameConfig,
        hand::Hand,
    },
};

#[derive(Debug, Clone)]
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

//#[derive(Clone)]
pub struct Game {
    card_source: Rc<RefCell<dyn CardSource>>,
    pub dealer_hand: Hand,
    pub player_hand: Hand,
    result: Option<GameResult>,
    double_down: bool,
    config: GameConfig,
}

impl Default for Game {
    fn default() -> Self {
        Game::with_deck(RandomDeck)
    }
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }

    fn with_deck(mut deck: impl CardSource + 'static) -> Self {
        Self {
            dealer_hand: Hand {
                cards: vec![deck.draw()],
            },
            player_hand: Hand {
                cards: vec![deck.draw(), deck.draw()],
            },
            result: None,
            double_down: false,
            config: GameConfig::default(),
            card_source: Rc::new(RefCell::new(deck)),
        }
    }
}

impl Game {
    pub fn from_split(game: &Game, card: &Card) -> Self {
        Self {
            dealer_hand: game.dealer_hand.clone(),
            player_hand: Hand {
                cards: vec![*card, game.card_source.borrow_mut().draw()],
            },
            result: None,
            double_down: false,
            config: game.config,
            card_source: game.card_source.clone(),
        }
    }

    pub fn game_loop(mut self) -> f32 {
        self.ai_loop();

        if let Some(GameResult::Split) = &self.result {
            let cards = self.player_hand.cards.clone();
            let score_a = Game::from_split(&self, &cards[0]).game_loop();
            let score_b = Game::from_split(&self, &cards[1]).game_loop();
            return score_a + score_b;
        }

        self.dealer_loop();

        println!("Outcome: {:?}", self.result);
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
                Action::Hit => self.player_hand.cards.push(self.card_source.borrow_mut().draw()),
                Action::Stand => return,
                Action::DoubleDown => {
                    self.double_down = true;
                    self.player_hand.cards.push(self.card_source.borrow_mut().draw());
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

            if self.player_hand.calc_points_best_possible() == 21 {
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
        while (self.config.action_on_17 == Soft17Rule::Stand && self.dealer_hand.calc_points_best_possible() < 17)
            || (self.config.action_on_17 == Soft17Rule::Hit && self.dealer_hand.calc_points_best_possible() <= 17)
        {
            self.dealer_hand.cards.push(self.card_source.borrow_mut().draw());
        }
    }

    fn calculate_outcome(&mut self) -> f32 {
        if self.result.is_none() {
            if self.dealer_hand.is_bust() {
                self.result = Some(GameResult::DealerBust);
            } else if self.dealer_hand.calc_points_best_possible() == self.player_hand.calc_points_best_possible() {
                if self.player_hand.is_natural_blackjack() && !self.dealer_hand.is_natural_blackjack() {
                    self.result = Some(GameResult::Win);
                } else if !self.player_hand.is_natural_blackjack() && self.dealer_hand.is_natural_blackjack() {
                    //This could be checked earlier
                    self.result = Some(GameResult::DealerWin);
                } else {
                    self.result = Some(GameResult::Tie);
                }
            } else if self.dealer_hand.calc_points_best_possible() < self.player_hand.calc_points_best_possible() {
                self.result = Some(GameResult::Win);
            } else {
                self.result = Some(GameResult::DealerWin)
            }
        }

    fn calculate_outcome(&self) -> f32 {
        self.result
            .as_ref()
            .expect("Why dafaq is game not done at the end?")
            .score()
            * if self.double_down { 2.0 } else { 1.0 }
    }
}
