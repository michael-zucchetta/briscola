use ansi_term::Colour::{Yellow, Purple, Green, Blue};

use crate::card;

pub trait Painter {
    fn print_card(card: card::Card);
}
