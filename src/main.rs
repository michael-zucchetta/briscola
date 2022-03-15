
use briscola::game;
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use briscola::console;
#[cfg(target_arch = "wasm32")]
use briscola::run_app;


#[cfg(target_arch = "wasm32")]
fn hello() {
        println!("with wasm");
}

#[cfg(not(target_arch = "wasm32"))]
fn hello() {
        println!("not wasm");
}


pub fn main() {
    println!("OHI");
    hello(); 
    #[cfg(not(target_arch = "wasm32"))]
    let game = game::Game::new(
       game::PlayersSize::Two,
       // game::GameMode::AIVsAI,
       game::GameMode::PlayerVsAI,
       console::ConsolePainter::new(),
       console::Console::new()
    );
    #[cfg(not(target_arch = "wasm32"))]
    let player_won = game.game();
    #[cfg(not(target_arch = "wasm32"))]
    println!("Player {} won", player_won);
    /* for card in deck.cards.borrow().iter() {
        println!("CICCIO CANE {}", card);
    }*/
    #[cfg(target_arch = "wasm32")]
    run_app();
}
