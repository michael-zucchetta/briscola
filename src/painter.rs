use ansi_term::Colour::{Yellow, Purple, Green, Blue};

use crate::card;
use crate::deck;
use crate::hand;

pub trait Painter {
    fn print_card(card: card::Card);

    fn draw_beginning(&self, deck: &deck::Deck);

    // fn draw_hands(hand1: hand::Hand, hand2::Hand);

    fn print_cards(&self, hand: &hand::Hand, player: usize);

    fn update_game();

    fn player_played_card(card: card::Card, player: usize);

    fn player_won(player: usize, cards: &Vec<card::Card>);

    fn player_scores(score1: u8, score2: u8);
}
