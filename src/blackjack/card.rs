use rand::{
    RngExt,
    distr::{Distribution, StandardUniform},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Card {
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

    pub fn is_ten_value(&self) -> bool {
        matches!(self, Card::Ten | Card::Jack | Card::Queen | Card::King)
    }
}

impl Distribution<Card> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Card {
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
}
