

use briscola::deck;
use briscola::game;
use briscola::console;
use briscola::run_app;

fn main() {
    let deck = deck::Deck::new();
    let game = game::Game::new(
       game::PlayersSize::Two,
       game::GameMode::AIvsAI,
       console::ConsolePainter::new(),
       console::Console::new()
    );
    let player_won = game.game();
    println!("Player {} won", player_won);
    /* for card in deck.cards.borrow().iter() {
        println!("CICCIO CANE {}", card);
    }*/

    run_app();
}
