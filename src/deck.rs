use crate::card;
use crate::hand;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::cell::RefCell;

pub struct Deck {
    cards: RefCell<Vec<card::Card>>,
}

impl Deck {
    pub fn new() -> Deck {
        let mut cards = card::Card::load_cards();
        Deck::shuffle_deck(&mut cards);
        // cloned -> clones the inner reference

        Deck {
            cards: RefCell::new(cards),
        }
    }

    pub fn get_briscola(&self) -> card::Card {
        self.cards.borrow().last().cloned().unwrap()
    }

    pub fn is_deck_empty(&self) -> bool {
        self.cards.borrow().len() == 0
    }

    fn shuffle_deck(cards: &mut Vec<card::Card>) {
        let mut rng = thread_rng();
        cards.shuffle(&mut rng);
    }

    pub fn get_card(&self) -> Option<card::Card> {
        let mut cards = self.cards.borrow_mut();
        if cards.is_empty() {
            None
        } else {
            Some(cards.remove(0))
        }
    }

    pub fn get_hand(&self) -> hand::Hand {
        let cards_hand = (0..2).map(|_| self.cards.borrow_mut().remove(0)).collect();
        let hand = hand::Hand::new();
        hand.assign_cards(cards_hand);
        hand
    }
}
