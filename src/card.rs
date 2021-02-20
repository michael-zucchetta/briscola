
use self::CardSuit::*;
use self::CardNumber::*;
use std::slice::Iter;

// allows printing
#[derive(Debug, Clone, Copy)]
pub enum CardSuit {
    Cups,
    Batons,
    Coins,
    Swords
}

impl CardSuit {
    pub fn iterator() -> Iter<'static, CardSuit> {
	static CARD_SUITS: [CardSuit; 4] = [Cups, Batons, Coins, Swords];
	CARD_SUITS.iter()
    }
}

#[derive(Clone, Copy)]
pub enum CardNumber {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Knave, // Fante
    Knight, // Cavallo
    King, //
}

impl CardNumber {
    pub fn iterator() -> Iter<'static, CardNumber> {
        static CARD_NUMBERS: [CardNumber; 10] = [Ace, Two, Three, Four, Five, Six, Seven, Knave, Knight, King];
        CARD_NUMBERS.iter()
    }
}

pub struct Card {
    value: CardNumber,
    suit: CardSuit
}

impl Card {
    pub fn new(value: CardNumber, suit: CardSuit) -> Card {
	Card {
	    value,
	    suit,
	}
    }

    fn load_cards() -> Vec<Card> {// &'static [Card] {
        CardSuit::iterator().flat_map(move |&suit| {
            CardNumber::iterator().map(move |&number|
                 Card::new(number, suit)
            )
        }).collect::<Vec<Card>>()
    }
}
