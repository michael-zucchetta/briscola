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
    user_input: T
}

impl <T> Player <T> where T: game::UserInput {
    pub fn new(player_type: PlayerType, user_input: T) -> Player<T> {
	Player {
	    player_type: player_type,
	    hand: Cell::new(vec![]),
	    cards_won: vec![],
            user_input: user_input,
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
    use crate::console;
    #[test]
    fn select_card() {
	let card1 = card::Card::new( card::CardNumber::Two, card::CardSuit::Cups);
	let card2 = card::Card::new( card::CardNumber::Four, card::CardSuit::Cups);
	let card3 = card::Card::new( card::CardNumber::King, card::CardSuit::Swords);
        let mut hand = Vec::with_capacity(3);
        hand.push(card1);
        hand.push(card2);
        hand.push(card3);
        let player = Player::new(PlayerType::Player, console::Console::new());
        player.assign_cards(hand);
        let selected_card = player.select_card(2);
        assert_eq!(selected_card, card2);
    }
}

