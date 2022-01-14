

use briscola::deck;
use briscola::game;
use briscola::console;

fn main() {
    let deck = deck::Deck::new();
    let game = game::Game::new(
       game::PlayersSize::Two,
       game::GameMode::AIvsAI,
       console::ConsolePainter::new(),
       console::Console::new()
    );
    game.game();
    /* for card in deck.cards.borrow().iter() {
        println!("CICCIO CANE {}", card);
    }*/


}
