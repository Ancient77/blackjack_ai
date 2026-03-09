use blackjack_ai::{blackjack::game::Game, player::command_line_user::CommandLineUser};
fn main() {
    let _score = Game::new(CommandLineUser).game_loop();
}
