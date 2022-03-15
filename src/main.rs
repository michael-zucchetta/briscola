

use briscola::deck;
use briscola::game;
use briscola::console;
#[cfg(target_arch = "wasm32")]
use briscola::run_app;
use std::env;

fn main() {
    let deck = deck::Deck::new();
    let game = game::Game::new(
       game::PlayersSize::Two,
       // game::GameMode::AIVsAI,
       game::GameMode::PlayerVsAI,
       console::ConsolePainter::new(),
       console::Console::new()
    );
    let player_won = game.game();
    println!("Player {} won", player_won);
    /* for card in deck.cards.borrow().iter() {
        println!("CICCIO CANE {}", card);
    }*/
    #[cfg(target_arch = "wasm32")]
    run_app();
}
