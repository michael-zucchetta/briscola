use crate::card;

use std::cell::Cell;

pub enum PlayerType {
    AI,
    Player
}

pub struct Player {
    player_type: PlayerType,
    hand: Cell<Vec<card::Card>>,
    cards_won: Vec<card::Card>
}

impl Player {
    pub fn new(player_type: PlayerType) -> Player {
	Player {
	    player_type: player_type,
	    hand: Cell::new(vec![]),
	    cards_won: vec![],
	}
    }

    pub fn assign_cards(&self, cards: Vec<card::Card>) {
	self.hand.set(cards);
    }

    pub fn select_card(&self, selected_card: usize) -> card::Card {
        let card_index = selected_card - 1usize; 
	let cards = self.hand.as_ptr();
	// check if it exists
	unsafe {
	    (*cards).remove(card_index)
	}
    }
}

#[cfg(test)]
mod tests {
    use crate::player::*;
    #[test]
    fn select_card() {
	let card1 = card::Card::new( card::CardNumber::Two, card::CardSuit::Cups);
	let card2 = card::Card::new( card::CardNumber::Four, card::CardSuit::Cups);
	let card3 = card::Card::new( card::CardNumber::King, card::CardSuit::Swords);
        let mut hand = Vec::with_capacity(3);
        hand.push(card1);
        hand.push(card2);
        hand.push(card3);
        let player = Player::new(PlayerType::Player);
        player.assign_cards(hand);
        let selected_card = player.select_card(2);
        assert_eq!(selected_card, card2); 
    }
}

