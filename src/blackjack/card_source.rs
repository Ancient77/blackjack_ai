use rand::RngExt;

use crate::blackjack::card::Card;

pub trait CardSource {
    fn draw(&mut self) -> Card;
}

pub struct RandomDeck;

impl CardSource for RandomDeck {
    fn draw(&mut self) -> Card {
        rand::rng().random()
    }
}

#[cfg(test)]
pub struct FixedDeck {
    cards: Vec<Card>,
    index: usize,
}

#[cfg(test)]
impl FixedDeck {
    pub fn new(cards: Vec<Card>) -> Self {
        Self { cards, index: 0 }
    }
}

#[cfg(test)]
impl CardSource for FixedDeck {
    fn draw(&mut self) -> Card {
        let card = self.cards[self.index];
        self.index += 1;
        card
    }
}
