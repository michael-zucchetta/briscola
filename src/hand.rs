use crate::card;

use std::cell::Cell;

pub struct Hand {
  cards: Cell<Vec<card::Card>>,
  size: Cell<usize>,
}

impl Hand {
  pub fn assign_cards(&self, cards: Vec<card::Card>) {
    println!("Cards are {:?}", cards);
    let size = cards.len();
    self.cards.set(cards);
    self.size.set(size);
  }

  pub fn select_card(&self, selected_card: usize) -> card::Card {
    let card_index = selected_card;
    let cards = self.cards.take();
    println!("Cards are {:?}", cards);
    println!("selected card {}", selected_card);
    let chosen_card = cards.get(selected_card).unwrap();
    let filtered_cards = cards.iter().enumerate().filter(|(index, element)| {
       *index != card_index
    })
    .map(|(index, element)| element.clone())
    .collect::<Vec<card::Card>>();
    self.size.set(filtered_cards.len());
    self.cards.set(filtered_cards);
    // unstabvle self.size.update(|size| size - 1usize);
    chosen_card.clone()
    // cards_mut.get_mut().remove(card_index)
    /*self.cards.set(cards_mut);
    // check if it exists
    unsafe {
        (*cards).remove(card_index)
    }*/
  }

  pub fn get_hand(&self) -> Vec<card::Card> {
    self.cards.take()
  }

  pub fn size(&self) -> usize {
    self.size.clone().take()
  }

  pub fn new () -> Hand {
    Hand {
      cards: Cell::new(vec![]),
      size: Cell::new(0usize),
    }
  }
}
