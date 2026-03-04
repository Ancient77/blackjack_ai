use crate::blackjack::card::Card;

#[derive(Debug, Clone)]
pub struct Hand {
    pub cards: Vec<Card>,
}

impl Hand {
    pub fn is_natural_blackjack(&self) -> bool {
        self.cards.len() == 2 && self.cards.contains(&Card::Ace) && self.cards.iter().any(|c| c.is_ten_value())
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
