use rand::Rng;
use rand::seq::SliceRandom;
use crate::card;

pub struct Deck {
    pub cards: Vec<card::Card>
}

impl Deck {
    pub fn new() -> Deck {
        let mut cards = card::Card::load_cards();
        Deck::shuffle_deck(&mut cards);
        Deck{
            cards: cards 
        }
    }

    fn shuffle_deck(cards: &mut Vec<card::Card>) { 
        cards.shuffle(&mut rand::thread_rng());
    }
}
