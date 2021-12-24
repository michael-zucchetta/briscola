use crate::card;

use std::cell::Cell;
use crate::game;
use crate::hand;



#[derive(PartialEq, Clone, Copy)]
pub enum PlayerType {
    AI,
    Player
}

pub struct Player<T: game::UserInput>  {
    player_type: PlayerType,
    hand: hand::Hand,
    cards_won: Cell<Vec<card::Card>>,
    input_handler: T
}

impl <T> Player <T> where T: game::UserInput {
    pub fn new(player_type: PlayerType, user_input: T) -> Player<T> {
	Player {
	    player_type: player_type,
	    hand: hand::Hand::new(),
	    cards_won: Cell::new(vec![]),
            input_handler: user_input,
	}
    }

    pub fn assign_cards(&self, cards: Vec<card::Card>) {
	self.hand.assign_cards(cards);
    }

    pub fn select_card(&self, selected_card: usize) -> card::Card {
        self.hand.select_card(selected_card)
    }

    pub fn play_card(&self) -> card::Card {
       let selected = if self.player_type == PlayerType::AI {
           0
       } else {
           self.input_handler.user_input()
       };
       self.select_card(selected)
    }

    pub fn add_won_cards(&self, cards: &mut Vec<card::Card>) {
        let mut cards_won = self.cards_won.take();
        cards_won.append(cards);
        self.cards_won.set(cards_won);
    }

    pub fn calculate_score(&self) -> u8 {
        let cards_won = self.cards_won.take();
        cards_won.into_iter().map(|card| {
          match card.value.eval() {
            1u8 => 11u8,
            3u8 => 10u8,
            8u8 => 2u8,
            9u8 => 3u8,
            10u8 => 4u8,
            _ => 0u8,
          }
        }).sum()
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

    #[test]
    fn add_won_cards() {
	let card1 = card::Card::new( card::CardNumber::Two, card::CardSuit::Cups);
	let card2 = card::Card::new( card::CardNumber::Four, card::CardSuit::Cups);
	let card3 = card::Card::new( card::CardNumber::King, card::CardSuit::Swords);
        let mut player = Player::new(PlayerType::Player, console::Console::new());
        let mut hand = Vec::with_capacity(3);
        hand.push(card1);
        hand.push(card2);
        hand.push(card3);
        let hand_stored = hand.clone();
        player.add_won_cards(&mut hand);
        assert_eq!(player.cards_won.take(), hand_stored);
    }

    #[test]
    fn calculate_score() {
	let card1 = card::Card::new( card::CardNumber::Two, card::CardSuit::Cups);
	let card2 = card::Card::new( card::CardNumber::Four, card::CardSuit::Cups);
	let card3 = card::Card::new( card::CardNumber::King, card::CardSuit::Swords);
        let mut player = Player::new(PlayerType::Player, console::Console::new());
        let mut hand = Vec::with_capacity(3);
        hand.push(card1);
        hand.push(card2);
        hand.push(card3);
        player.add_won_cards(&mut hand);
        assert_eq!(player.calculate_score(), 4u8);
    }
}

