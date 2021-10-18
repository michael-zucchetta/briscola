

use briscola::deck;
use briscola::game;

fn main() {
    let deck = deck::Deck::new();

    // let game = game::Game(game::PlayersSize::Two, game::GameMode::PlayerVsPlayer, );
    
    for card in deck.cards.borrow().iter() {
        println!("CICCIO CANE {}", card);
    }
}
