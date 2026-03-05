#[derive(Copy, Clone)]
pub struct GameConfig {
    pub action_on_17: Soft17Rule,
    pub peeking: HoleCardRule,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            action_on_17: Soft17Rule::Stand,
            peeking: HoleCardRule::Peek,
        }
    }
}

/// Whether the dealer hits or stands on soft 17.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Soft17Rule {
    Stand,
    Hit,
}

/// Whether the dealer peeks at the hole card for natural blackjack.
#[derive(Clone, Copy)]
pub enum HoleCardRule {
    //If dealer gets Ace or 10, they reveal other card early
    Peek,
    //dealer doesn't peek second card
    NoPeek,
}
