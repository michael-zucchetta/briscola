
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

    pub fn eval(&self) -> u8 {
        match self {
            Ace => 1u8,
            Two => 2u8,
            Three => 3u8,
            Four => 4u8,
            Five => 5u8,
            Six => 6u8,
            Seven => 7u8,
            Knave => 8u8,
            Knight => 9u8,
            King => 10u8
        }
    }
}

#[derive(Clone, Copy)]
pub struct Card {
    pub value: CardNumber,
    pub suit: CardSuit
}

impl Card {
    pub fn new(value: CardNumber, suit: CardSuit) -> Card {
	Card {
	    value,
	    suit,
	}
    }

    pub fn load_cards() -> Vec<Card> {// &'static [Card] {
        CardSuit::iterator().flat_map(move |&suit| {
            CardNumber::iterator().map(move |&number|
                 Card::new(number, suit)
            )
        }).collect::<Vec<Card>>()
    }
}
