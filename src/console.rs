use ansi_term::Colour::{Yellow, Red, Green, Blue};
use ansi_term::Colour;

use crate::painter;
use crate::card;
use crate::deck;
use crate::hand;
use crate::game;

use std::fmt;
use std::io;
use std::string;

pub struct ConsolePainter {
}


impl ConsolePainter {
    fn get_card(card: card::Card) -> (u8, &'static str, Colour) {
        let (color, suit_as_string) = match card.suit {
            card::CardSuit::Cups => (Blue, "Cups"),
            card::CardSuit::Batons => (Green, "Batons"),
            card::CardSuit::Coins => (Yellow, "Coins"),
            card::CardSuit::Swords => (Red, "Swords"),
        };
        let value = card.value.eval();
        (value, suit_as_string, color)
    }

    pub fn new() -> ConsolePainter {
       ConsolePainter {
       }
    }
}

impl painter::Painter for ConsolePainter {
    fn print_card(card: card::Card) {
        let (value, suit, color) = ConsolePainter::get_card(card);
        println!("{} {}", value, color.bold().paint(suit));
    }

    fn draw_beginning(deck: &deck::Deck) {
      println!("Beginning game");
      println!("Briscola is {}", deck.get_briscola());
    }

    fn update_game() {
    }
}

impl fmt::Display for card::Card  {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (value, suit, color) = ConsolePainter::get_card(*self);
        write!(f, "{} {}", value, color.bold().paint(suit))
    }
}

impl fmt::Display for hand::Hand  {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hand = self.get_hand();
        let resulting_string = hand.into_iter().map(|card| {
            let (value, suit, color) = ConsolePainter::get_card(card);
            format!("{} {}", value, color.bold().paint(suit))
        }).fold("".to_string(), |base, new_fmt_card| {
          format!("{}  {}", base, new_fmt_card)
        });
        write!(f, "{}", resulting_string)
    }
}


#[derive(Clone)]
pub struct Console {

}

impl Console {

  pub fn new() -> Console {
    Console {}
  }
}

impl game::UserInput for Console {
    fn user_input(&self) -> usize {
        let mut command_as_text = string::String::new();
        println!("selecting move");
        io::stdin().read_line(&mut command_as_text);
        println!("selecting move");
        command_as_text.parse::<usize>().unwrap()
    }
}


impl fmt::Display for game::Game<ConsolePainter, Console> {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
     write!(f, "")
   }

}

/*
impl game::LiveGame for player::Player {
    fn user_input(self) -> card::Card {
        let mut command_as_text = string::new();
        println!("selecting move");
        io::stdin().read_line(&mut command_as_text);
        println!("selecting move");
        let card_index = command_as_text.parse::<usize>().unwrap();
        self.select_card(card_index)
    }
}
*/

#[cfg(test)]
mod tests {
    use crate::console::*;
    #[test]
    fn get_card() {
        let card = card::Card::new( card::CardNumber::Two, card::CardSuit::Cups);
        let (value, suit, color) = ConsolePainter::get_card(card);
        assert_eq!(color, Colour::Blue);
        assert_eq!(value, 2u8);
        assert_eq!(suit, "Cups");
    }
}
