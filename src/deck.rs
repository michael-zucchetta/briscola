use rand::Rng;
use rand::seq::SliceRandom;
use crate::card;
use crate::hand;
use std::cell::RefCell;

pub struct Deck {
    pub cards: RefCell<Vec<card::Card>>,
    pub briscola: card::Card
}

impl Deck {
    pub fn new() -> Deck {
        let mut cards = card::Card::load_cards();
        Deck::shuffle_deck(&mut cards);
        // cloned -> clones the inner reference
        let briscola = cards.last().cloned().unwrap();

        Deck{
            cards: RefCell::new(cards),
            briscola: briscola,
        }
    }

    fn shuffle_deck(cards: &mut Vec<card::Card>) {
        cards.shuffle(&mut rand::thread_rng());
    }

    pub fn get_card(&self) -> Option<card::Card> {
        self.cards.borrow_mut().pop()
    }

    pub fn get_hand(&self) -> hand::Hand {
        let cards_hand = (0..2).map(|_| {
            self.cards.borrow_mut().pop().unwrap()
        }).collect();
        let hand = hand::Hand::new();
        hand.assign_cards(cards_hand);
        hand
    }
}
