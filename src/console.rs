use ansi_term::Colour::{Yellow, Red, Green, Blue};
use ansi_term::Colour;

use crate::painter;
use crate::card;

use std::fmt;

struct ConsolePainter {
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

 
}

impl painter::Painter for ConsolePainter {
    fn print_card(card: card::Card) {
        let (value, suit, color) = ConsolePainter::get_card(card);
        println!("{}{}", value, color.bold().paint(suit));
        
    }
}

impl fmt::Display for card::Card  {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (value, suit, color) = ConsolePainter::get_card(*self);
        write!(f, "{}{}", value, color.bold().paint(suit))
        
    }
}

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
