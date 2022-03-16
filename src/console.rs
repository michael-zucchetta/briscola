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
  game_mode: game::GameMode
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

    pub fn new(game_mode: game::GameMode) -> ConsolePainter {
       ConsolePainter {
           game_mode: game_mode,
       }
    }
}

impl painter::Painter for ConsolePainter {
    fn print_card(card: card::Card) {
        let (value, suit, color) = ConsolePainter::get_card(card);
        println!("{} {}", value, color.bold().paint(suit));
    }

    fn draw_beginning(&self, deck: &deck::Deck) {
      println!("Beginning game");
      println!("Briscola is {}", deck.get_briscola());
    }

    fn update_game() {
    }

    fn print_cards(hand: hand::Hand, player: usize) {
       println!("User hand is {:?}", hand.get_hand_ref());
    }

    fn player_played_card(card: card::Card, player: usize) {
       println!("Card played by player {} is {}", player, card);
    }

    fn player_won(player: usize, cards: &Vec<card::Card>) {
      println!("Player {} won turn, and won these cards {:?}", player, cards);
    }

    fn player_scores(score1: u8, score2: u8) {
        println!("Player 1 score is {}", score1);
        println!("Player 2 score is {}", score2);
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
    fn user_input(&self, hand: &hand::Hand) -> usize {
        let mut command_as_text = string::String::new();
        let hand_size = hand.size();
        println!("selecting move. press p to print hand");
        io::stdin().read_line(&mut command_as_text);
        let selected_move = command_as_text.trim();
        if selected_move == "p" {
           println!("User hand is {:?}", hand.get_hand_ref());
           return self.user_input(hand)
        }
        println!("selected move {}", selected_move);
        let chosen_card = selected_move.parse().unwrap();
        if chosen_card < hand_size {
            chosen_card
        } else {
            println!("Selected move, {}, is wrong as hand size is {}", chosen_card, hand_size);
            self.user_input(hand)
        }
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
