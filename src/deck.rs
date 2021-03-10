use rand::Rng;
use rand::seq::SliceRandom;
use crate::card;
use std::cell::RefCell;

pub struct Deck {
    pub cards: RefCell<Vec<card::Card>>
}

impl Deck {
    pub fn new() -> Deck {
        let mut cards = card::Card::load_cards();
        Deck::shuffle_deck(&mut cards);
        Deck{
            cards: RefCell::new(cards) 
        }
    }

    fn shuffle_deck(cards: &mut Vec<card::Card>) { 
        cards.shuffle(&mut rand::thread_rng());
    }

    pub fn get_card(&self) -> Option<card::Card> {
        self.cards.borrow_mut().pop()
    }
}
