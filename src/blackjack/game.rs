use std::{cell::RefCell, rc::Rc};

use crate::{
    blackjack::{
        Action,
        card::Card,
        card_source::{CardSource, RandomDeck},
        game_config::{GameConfig, Soft17Rule},
        hand::Hand,
    },
    player::Player,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum GameResult {
    Bust,
    DealerBust,
    Win,
    DealerWin,
    Tie,
    Surrender,
    Split,
    BlackjackWin,
}

impl GameResult {
    pub fn score(&self) -> f32 {
        match self {
            GameResult::Bust | GameResult::DealerWin => -1.0,
            GameResult::Tie => 0.0,
            GameResult::Surrender => -0.5,
            GameResult::Win | GameResult::DealerBust => 1.0,
            GameResult::Split => unreachable!("split is resolved in play()"),
            GameResult::BlackjackWin => 1.5,
        }
    }
}

pub struct Game {
    card_source: Rc<RefCell<dyn CardSource>>,
    player: Rc<RefCell<dyn Player>>,
    pub dealer_hand: Hand,
    pub player_hand: Hand,
    result: Option<GameResult>,
    double_down: bool,
    insurance: bool,
    config: GameConfig,
}

impl Game {
    pub fn new(user: impl Player + 'static) -> Self {
        let mut deck = RandomDeck;
        Self {
            dealer_hand: Hand {
                cards: vec![deck.draw()],
            },
            player_hand: Hand {
                cards: vec![deck.draw(), deck.draw()],
            },
            result: None,
            double_down: false,
            insurance: false,
            config: GameConfig::default(),
            card_source: Rc::new(RefCell::new(deck)),
            player: Rc::new(RefCell::new(user)),
        }
    }

    #[cfg(test)]
    fn with_deck(
        deck: impl CardSource + 'static,
        user: impl Player + 'static,
        dealer_hand: Hand,
        player_hand: Hand,
    ) -> Self {
        Self {
            dealer_hand,
            player_hand,
            result: None,
            double_down: false,
            insurance: false,
            config: GameConfig::default(),
            card_source: Rc::new(RefCell::new(deck)),
            player: Rc::new(RefCell::new(user)),
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
            insurance: false,
            config: game.config,
            card_source: game.card_source.clone(),
            player: game.player.clone(),
        }
    }

    pub fn game_loop(&mut self) -> f32 {
        self.player_loop();

        if let Some(GameResult::Split) = &self.result {
            let cards = self.player_hand.cards.clone();
            let score_a = Game::from_split(self, &cards[0]).game_loop();
            let score_b = Game::from_split(self, &cards[1]).game_loop();
            return score_a + score_b;
        }

        self.dealer_loop();

        self.calculate_outcome()
    }

    fn player_loop(&mut self) {
        let mut legal_moves = vec![Action::Hit, Action::Stand, Action::DoubleDown, Action::Surrender];

        if self.dealer_hand.cards.contains(&Card::Ace) {
            legal_moves.push(Action::Insurance);
        }

        if self.player_hand.is_natural_blackjack() {
            //TODO: Even money: If Dealer has Ace, Player can quit here with 1:1 Payout
            // (mathematically its the same as Insurance and always bad)
            return;
        }

        if self.player_hand.cards[0].card_to_score_ace_as_var(11)
            == self.player_hand.cards[1].card_to_score_ace_as_var(11)
        {
            legal_moves.push(Action::Split);
        }

        loop {
            match self.player.borrow_mut().ask_user(self, &legal_moves) {
                Action::Hit => {
                    self.player_hand.cards.push(self.card_source.borrow_mut().draw());
                    legal_moves.retain(|&x| x != Action::DoubleDown);
                }
                Action::Stand => return,
                Action::DoubleDown => {
                    self.double_down = true;
                    self.player_hand.cards.push(self.card_source.borrow_mut().draw());
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
                Action::Insurance => {
                    self.insurance = true;
                }
            }

            // Remove Insurance, Split & Surrender Option
            legal_moves.retain(|&x| {
                x != Action::Insurance && x != Action::Split && x != Action::Surrender
            });


            if self.player_hand.calc_points_best_possible() == 21 {
                return;
            }

            if self.player_hand.calc_points_ace_as_one() > 21 {
                return;
            }
        }
    }

    fn dealer_loop(&mut self) {
        if self.result.is_some() {
            panic!("Dealer Loop started, result should be none.")
        }

        //Hit until 17
        while (self.dealer_hand.calc_points_best_possible() < 17)
            || (self.config.action_on_17 == Soft17Rule::Hit && self.dealer_hand.is_soft_17())
        {
            self.dealer_hand.cards.push(self.card_source.borrow_mut().draw());
        }
    }

    fn calculate_result(&mut self) -> GameResult {
        if self.result.is_some() {
            panic!("calculate_outcome started, result should be none.")
        }

        if self.player_hand.is_natural_blackjack() && !self.dealer_hand.is_natural_blackjack() {
            return GameResult::BlackjackWin;
        }

        if self.player_hand.is_bust() {
            return GameResult::Bust;
        }

        if self.dealer_hand.is_bust() {
            return GameResult::DealerBust;
        }

        if self.dealer_hand.calc_points_best_possible() == self.player_hand.calc_points_best_possible() {
            if !self.player_hand.is_natural_blackjack() && self.dealer_hand.is_natural_blackjack() {
                return GameResult::DealerWin;
            }

            return GameResult::Tie;
        }

        if self.dealer_hand.calc_points_best_possible() < self.player_hand.calc_points_best_possible() {
            return GameResult::Win;
        }

        GameResult::DealerWin
    }

    fn calculate_outcome(&mut self) -> f32 {
        if self.result.is_some() {
            panic!("calculate_outcome started, result should be none.")
        }

        self.result = Some(self.calculate_result());

        let mut insurance_result = 0.0;
        if self.insurance {
            if self.dealer_hand.is_natural_blackjack() {
                insurance_result = 1.0;
            } else {
                insurance_result = -0.5;
            }
        }

        self.result
            .as_ref()
            .expect("Why dafaq is game not done at the end?")
            .score()
            * if self.double_down { 2.0 } else { 1.0 }
            + insurance_result
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        blackjack::{card_source::FixedDeck, game_config::Soft17Rule},
        player::test_user::TestUser,
    };

    use super::*;

    #[test]
    fn dealer_bust() {
        let deck = FixedDeck::new(vec![Card::Jack]);
        let mut game = Game::with_deck(
            deck,
            TestUser::default(),
            Hand {
                cards: vec![Card::King, Card::Two],
            },
            Hand { cards: vec![] },
        );
        game.dealer_loop();
        assert!(game.dealer_hand.cards.contains(&Card::Jack));

        let outcome = game.calculate_outcome();
        assert_eq!(game.result.unwrap(), GameResult::DealerBust);
        assert_eq!(outcome, 1.0);
    }

    #[test]
    fn dealer_hit_until_17() {
        let deck = FixedDeck::new(vec![Card::King, Card::Three, Card::Two, Card::Nine]);
        let mut game = Game::with_deck(
            deck,
            TestUser::default(),
            Hand { cards: vec![] },
            Hand { cards: vec![] },
        );
        game.dealer_loop();

        assert_eq!(
            game.dealer_hand.cards,
            vec![Card::King, Card::Three, Card::Two, Card::Nine]
        );
    }

    #[test]
    fn dealer_hit_on_17_when_soft_17() {
        let deck = FixedDeck::new(vec![Card::Six, Card::Ace, Card::Ace]);
        let mut game = Game::with_deck(
            deck,
            TestUser::default(),
            Hand { cards: vec![] },
            Hand { cards: vec![] },
        );
        game.config.action_on_17 = Soft17Rule::Hit;
        game.dealer_loop();

        assert_eq!(game.dealer_hand.cards, vec![Card::Six, Card::Ace, Card::Ace]);
    }

    #[test]
    fn dealer_stand_on_17_when_soft_17() {
        let deck = FixedDeck::new(vec![Card::Six, Card::Ace, Card::Ace]);
        let mut game = Game::with_deck(
            deck,
            TestUser::default(),
            Hand { cards: vec![] },
            Hand { cards: vec![] },
        );
        game.config.action_on_17 = Soft17Rule::Stand;
        game.dealer_loop();

        assert_eq!(game.dealer_hand.cards, vec![Card::Six, Card::Ace]);
    }

    #[test]
    fn dealer_stand_on_17_when_no_soft_17() {
        let deck = FixedDeck::new(vec![Card::Five, Card::Two, Card::Ace, Card::Two]);
        let mut game = Game::with_deck(
            deck,
            TestUser::default(),
            Hand { cards: vec![] },
            Hand { cards: vec![] },
        );
        game.config.action_on_17 = Soft17Rule::Hit;
        game.dealer_loop();

        assert_eq!(game.dealer_hand.cards, vec![Card::Five, Card::Two, Card::Ace]);
    }

    #[test]
    fn player_should_bust_over_21() {
        let deck = FixedDeck::new(vec![Card::Four, Card::Five, Card::Six, Card::Seven, Card::Eight]);
        let test_user = TestUser::new(vec![Action::Hit; 10]);
        let mut game = Game::with_deck(
            deck,
            test_user,
            Hand { cards: vec![] },
            Hand {
                cards: vec![Card::Two, Card::Three],
            },
        );
        game.player_loop();

        assert_eq!(
            game.player_hand.cards,
            vec![Card::Two, Card::Three, Card::Four, Card::Five, Card::Six, Card::Seven]
        );
        game.calculate_outcome();
        assert_eq!(game.result.unwrap(), GameResult::Bust);
    }

    #[test]
    fn player_should_be_able_double_down_and_win() {
        let deck = FixedDeck::new(vec![Card::Ace, Card::Five, Card::Seven]);
        let test_user = TestUser::new(vec![Action::DoubleDown]);
        let mut game = Game::with_deck(
            deck,
            test_user,
            Hand {
                cards: vec![Card::Five],
            },
            Hand {
                cards: vec![Card::Five, Card::Five],
            },
        );
        let result = game.game_loop();

        assert_eq!(game.player_hand.cards, vec![Card::Five, Card::Five, Card::Ace]);
        assert_eq!(game.dealer_hand.cards, vec![Card::Five, Card::Five, Card::Seven]);
        assert_eq!(game.result.unwrap(), GameResult::Win);
        assert_eq!(result, 2.0)
    }

    #[test]
    fn player_should_be_able_double_down_and_lose() {
        let deck = FixedDeck::new(vec![Card::Ten, Card::Five, Card::Ace]);
        let test_user = TestUser::new(vec![Action::DoubleDown]);
        let mut game = Game::with_deck(
            deck,
            test_user,
            Hand {
                cards: vec![Card::Five],
            },
            Hand {
                cards: vec![Card::Five, Card::Five],
            },
        );
        let result = game.game_loop();

        assert_eq!(game.player_hand.cards, vec![Card::Five, Card::Five, Card::Ten]);
        assert_eq!(game.dealer_hand.cards, vec![Card::Five, Card::Five, Card::Ace]);
        assert_eq!(game.result.unwrap(), GameResult::DealerWin);
        assert_eq!(result, -2.0)
    }

    #[test]
    fn player_should_be_able_double_down_and_tie() {
        let deck = FixedDeck::new(vec![Card::Ace, Card::Five, Card::Ace]);
        let test_user = TestUser::new(vec![Action::DoubleDown]);
        let mut game = Game::with_deck(
            deck,
            test_user,
            Hand {
                cards: vec![Card::Five],
            },
            Hand {
                cards: vec![Card::Five, Card::Five],
            },
        );
        let result = game.game_loop();

        assert_eq!(game.player_hand.cards, vec![Card::Five, Card::Five, Card::Ace]);
        assert_eq!(game.dealer_hand.cards, vec![Card::Five, Card::Five, Card::Ace]);
        assert_eq!(game.result.unwrap(), GameResult::Tie);
        assert_eq!(result, 0.0)
    }

    #[test]
    fn player_buys_insurance_dealer_no_blackjack_hits_and_busts() {
        let deck = FixedDeck::new(vec![Card::Ten, Card::Nine]);
        let test_user = TestUser::new(vec![Action::Insurance, Action::Hit]);
        let mut game = Game::with_deck(
            deck,
            test_user,
            Hand { cards: vec![Card::Ace] },
            Hand {
                cards: vec![Card::Ten, Card::Five],
            },
        );
        let result = game.game_loop();

        assert_eq!(game.player_hand.cards, vec![Card::Ten, Card::Five, Card::Ten]);
        assert_eq!(game.result.unwrap(), GameResult::Bust);
        assert_eq!(result, -1.5);
    }

    #[test]
    fn player_buys_insurance_dealer_no_blackjack_player_doubles_down_and_wins() {
        let deck = FixedDeck::new(vec![Card::Ten, Card::Six]);
        let test_user = TestUser::new(vec![Action::Insurance, Action::DoubleDown]);
        let mut game = Game::with_deck(
            deck,
            test_user,
            Hand { cards: vec![Card::Ace] },
            Hand {
                cards: vec![Card::Six, Card::Five],
            },
        );
        let result = game.game_loop();

        assert_eq!(game.player_hand.cards, vec![Card::Six, Card::Five, Card::Ten]);
        assert_eq!(game.result.unwrap(), GameResult::Win);
        assert_eq!(result, 1.5);
    }

    #[test]
    fn player_declines_insurance_dealer_no_blackjack_player_hits_and_stands() {
        let deck = FixedDeck::new(vec![Card::Five, Card::Six, Card::Three]);
        let test_user = TestUser::new(vec![Action::Hit, Action::Stand]);
        let mut game = Game::with_deck(
            deck,
            test_user,
            Hand { cards: vec![Card::Ace] },
            Hand {
                cards: vec![Card::Ten, Card::Six],
            },
        );
        let result = game.game_loop();

        assert_eq!(game.player_hand.cards, vec![Card::Ten, Card::Six, Card::Five]);
        assert_eq!(game.result.unwrap(), GameResult::Win);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn player_declines_insurance_dealer_blackjack_player_hits_and_stands() {
        let deck = FixedDeck::new(vec![Card::Five, Card::Ten]);
        let test_user = TestUser::new(vec![Action::Hit, Action::Stand]);
        let mut game = Game::with_deck(
            deck,
            test_user,
            Hand { cards: vec![Card::Ace] },
            Hand {
                cards: vec![Card::Ten, Card::Six],
            },
        );
        let result = game.game_loop();

        assert_eq!(game.player_hand.cards, vec![Card::Ten, Card::Six, Card::Five]);
        assert_eq!(game.result.unwrap(), GameResult::DealerWin);
        assert_eq!(result, -1.0);
    }

    #[test]
    fn player_buys_insurance_plays_hand_then_dealer_reveals_blackjack() {
        let deck = FixedDeck::new(vec![Card::Two, Card::Ten]);
        let test_user = TestUser::new(vec![Action::Insurance, Action::Hit, Action::Stand]);
        let mut game = Game::with_deck(
            deck,
            test_user,
            Hand { cards: vec![Card::Ace] },
            Hand {
                cards: vec![Card::Ten, Card::Six],
            },
        );
        let result = game.game_loop();

        assert_eq!(game.player_hand.cards, vec![Card::Ten, Card::Six, Card::Two]);
        assert_eq!(game.dealer_hand.cards, vec![Card::Ace, Card::Ten]);
        assert_eq!(game.result.unwrap(), GameResult::DealerWin);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn player_wins_tree_to_two_on_blackjack() {
        let deck = FixedDeck::new(vec![Card::Ten]);
        let test_user = TestUser::new(vec![]);
        let mut game = Game::with_deck(
            deck,
            test_user,
            Hand { cards: vec![Card::Ten] },
            Hand {
                cards: vec![Card::Ace, Card::Jack],
            },
        );
        let result = game.game_loop();

        assert_eq!(game.player_hand.cards, vec![Card::Ace, Card::Jack]);
        assert_eq!(game.dealer_hand.cards, vec![Card::Ten, Card::Ten]);
        assert_eq!(game.result.unwrap(), GameResult::BlackjackWin);
        assert_eq!(result, 1.5);
    }

    #[test]
    fn player_wins_tree_to_two_on_blackjack_when_dealer_has_21() {
        let deck = FixedDeck::new(vec![Card::Six, Card::Five]);
        let test_user = TestUser::new(vec![]);
        let mut game = Game::with_deck(
            deck,
            test_user,
            Hand { cards: vec![Card::Ten] },
            Hand {
                cards: vec![Card::Ace, Card::Jack],
            },
        );
        let result = game.game_loop();

        assert_eq!(game.player_hand.cards, vec![Card::Ace, Card::Jack]);
        assert_eq!(game.dealer_hand.cards, vec![Card::Ten, Card::Six, Card::Five]);
        assert_eq!(game.result.unwrap(), GameResult::BlackjackWin);
        assert_eq!(result, 1.5);
    }

    #[test]
    fn player_loses_one_21_when_dealer_blackjack() {
        let deck = FixedDeck::new(vec![Card::Five, Card::Jack]);
        let test_user = TestUser::new(vec![Action::Hit]);
        let mut game = Game::with_deck(
            deck,
            test_user,
            Hand { cards: vec![Card::Ace] },
            Hand {
                cards: vec![Card::Ten, Card::Six],
            },
        );
        let result = game.game_loop();

        assert_eq!(game.player_hand.cards, vec![Card::Ten, Card::Six, Card::Five]);
        assert_eq!(game.dealer_hand.cards, vec![Card::Ace, Card::Jack]);
        assert_eq!(game.result.unwrap(), GameResult::DealerWin);
        assert_eq!(result, -1.0);
    }

    #[test]
    fn player_ties_on_both_blackjack() {
        let deck = FixedDeck::new(vec![Card::Jack]);
        let test_user = TestUser::new(vec![]);
        let mut game = Game::with_deck(
            deck,
            test_user,
            Hand { cards: vec![Card::Ace] },
            Hand {
                cards: vec![Card::Ace, Card::Jack],
            },
        );
        let result = game.game_loop();

        assert_eq!(game.player_hand.cards, vec![Card::Ace, Card::Jack]);
        assert_eq!(game.dealer_hand.cards, vec![Card::Ace, Card::Jack]);
        assert_eq!(game.result.unwrap(), GameResult::Tie);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn player_ties_on_both_21() {
        let deck = FixedDeck::new(vec![Card::Five, Card::Three, Card::Eight]);
        let test_user = TestUser::new(vec![Action::Hit, Action::Hit]);
        let mut game = Game::with_deck(
            deck,
            test_user,
            Hand { cards: vec![Card::Ten] },
            Hand {
                cards: vec![Card::Ten, Card::Six],
            },
        );
        let result = game.game_loop();

        assert_eq!(game.player_hand.cards, vec![Card::Ten, Card::Six, Card::Five]);
        assert_eq!(game.dealer_hand.cards, vec![Card::Ten, Card::Three, Card::Eight]);
        assert_eq!(game.result.unwrap(), GameResult::Tie);
        assert_eq!(result, 0.0);
    }

    //TODO: Test Surrender
}
