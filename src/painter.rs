use ansi_term::Colour::{Yellow, Purple, Green, Blue};

use crate::card;
use crate::deck;
use crate::hand;

pub trait Painter {
    fn print_card(card: card::Card);

    fn draw_beginning(deck: &deck::Deck);

    // fn draw_hands(hand1: hand::Hand, hand2::Hand);

    fn update_game();
}
