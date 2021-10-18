use crate::card;

use std::cell::Cell;
use crate::game;

pub enum PlayerType {
    AI,
    Player
}

pub struct Player<T: game::UserInput>  {
    player_type: PlayerType,
    hand: Cell<Vec<card::Card>>,
    cards_won: Vec<card::Card>,
    user_input_action: T
}

impl <T> Player <T> where T: game::UserInput {
    pub fn new(player_type: PlayerType, user_input_action: T) -> Player<T> {
	Player {
	    player_type: player_type,
	    hand: Cell::new(vec![]),
	    cards_won: vec![],
            user_input_action: user_input_action,
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

    pub fn hand_size(&self) -> usize {
        let cards = self.hand.as_ptr();
        unsafe { (*cards).len() }
    }
}

#[cfg(test)]
mod tests {
    use crate::player::*;

    #[derive(Clone)]
    struct MockInput {}

    impl game::UserInput for MockInput {
        fn user_input_action() -> usize {
            1usize
        }
    }

    #[test]
    fn select_card() {
	let card1 = card::Card::new( card::CardNumber::Two, card::CardSuit::Cups);
	let card2 = card::Card::new( card::CardNumber::Four, card::CardSuit::Cups);
	let card3 = card::Card::new( card::CardNumber::King, card::CardSuit::Swords);
        let mut hand = Vec::with_capacity(3);
        hand.push(card1);
        hand.push(card2);
        hand.push(card3);
        let player = Player::new(PlayerType::Player, MockInput{});
        player.assign_cards(hand);
        let selected_card = player.select_card(2);
        assert_eq!(selected_card, card2); 
    }
}

