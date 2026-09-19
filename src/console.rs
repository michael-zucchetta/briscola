use ansi_term::Colour;
use ansi_term::Colour::{Blue, Green, Red, Yellow};

use crate::card;
use crate::game;
use crate::hand;
use crate::painter;
use crate::player;

use std::fmt;
use std::io;
use std::string;

pub struct ConsolePainter {}

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
        ConsolePainter {}
    }
}

impl painter::Painter for ConsolePainter {
    fn print_card(card: card::Card) {
        let (value, suit, color) = ConsolePainter::get_card(card);
        println!("{} {}", value, color.bold().paint(suit));
    }

    fn draw_beginning() { // fn draw_beginning(deck: deck::Deck, players: [player::Player<Console>; 2]) {
    }

    fn update_game() {}
}

impl fmt::Display for card::Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (value, suit, color) = ConsolePainter::get_card(*self);
        write!(f, "{} {}", value, color.bold().paint(suit))
    }
}

impl fmt::Display for hand::Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hand = self.get_hand();
        let resulting_string = hand
            .into_iter()
            .map(|card| {
                let (value, suit, color) = ConsolePainter::get_card(card);
                format!("{} {}", value, color.bold().paint(suit))
            })
            .fold("".to_string(), |base, new_fmt_card| {
                format!("{}  {}", base, new_fmt_card)
            });
        write!(f, "{}", resulting_string)
    }
}

#[derive(Clone)]
pub struct Console {
    single_human_player: Option<usize>,
}

impl Console {
    pub fn new(player_types: [player::PlayerType; 2]) -> Console {
        let human_players = player_types
            .iter()
            .enumerate()
            .filter_map(|(index, player_type)| {
                if *player_type == player::PlayerType::Player {
                    Some(index + 1)
                } else {
                    None
                }
            })
            .collect::<Vec<usize>>();

        let single_human_player = if human_players.len() == 1 {
            human_players.first().copied()
        } else {
            None
        };

        Console {
            single_human_player,
        }
    }

    fn is_single_human_player(&self, player_index: usize) -> bool {
        self.single_human_player == Some(player_index)
    }

    fn cards_label(&self, player_index: usize) -> String {
        if self.is_single_human_player(player_index) {
            "Your cards:".to_string()
        } else {
            format!("Player {} cards:", player_index)
        }
    }

    fn selection_label(&self, player_index: usize) -> String {
        if self.is_single_human_player(player_index) {
            "Select your card index:".to_string()
        } else {
            format!("Select a card index for player {}:", player_index)
        }
    }
}

impl game::UserInput for Console {
    fn user_input(&self, player_index: usize, hand: &[card::Card]) -> usize {
        loop {
            println!("{}", self.cards_label(player_index));
            for (index, card) in hand.iter().enumerate() {
                println!("  [{}] {}", index, card);
            }
            println!("{}", self.selection_label(player_index));

            let mut command_as_text = string::String::new();
            let _ = io::stdin().read_line(&mut command_as_text);

            match command_as_text.trim().parse::<usize>() {
                Ok(selected) if selected < hand.len() => return selected,
                _ => println!("Invalid card index. Try again."),
            }
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
        let card = card::Card::new(card::CardNumber::Two, card::CardSuit::Cups);
        let (value, suit, color) = ConsolePainter::get_card(card);
        assert_eq!(color, Colour::Blue);
        assert_eq!(value, 2u8);
        assert_eq!(suit, "Cups");
    }

    #[test]
    fn single_human_uses_you_labels() {
        let console = Console::new([player::PlayerType::Player, player::PlayerType::AI]);

        assert_eq!(console.cards_label(1), "Your cards:");
        assert_eq!(console.selection_label(1), "Select your card index:");
    }

    #[test]
    fn two_humans_keep_player_labels() {
        let console = Console::new([player::PlayerType::Player, player::PlayerType::Player]);

        assert_eq!(console.cards_label(1), "Player 1 cards:");
        assert_eq!(
            console.selection_label(2),
            "Select a card index for player 2:"
        );
    }
}
