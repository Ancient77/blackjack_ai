mod card;
mod card_source;
pub mod game;
mod game_config;
mod hand;

// Difference between mutlicard 21 and natural 21 is not implemented
// Soft17 not implemented
// Even money not implemented
// Aces can be 11 or 1 in the same Hand
// Deck is infinite

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Hit,
    Stand,
    DoubleDown,
    Split,
    Surrender,
    Insurance,
}
