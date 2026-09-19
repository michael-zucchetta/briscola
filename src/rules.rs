/// Player number, or None when both scores are equal.
pub fn winner_from_scores(score1: u8, score2: u8) -> Option<usize> {
    match score1.cmp(&score2) {
        std::cmp::Ordering::Greater => Some(1),
        std::cmp::Ordering::Less => Some(2),
        std::cmp::Ordering::Equal => None,
    }
}

use crate::card;

pub fn card_points(card: card::Card) -> u8 {
    match card.value {
        card::CardNumber::Ace => 11,
        card::CardNumber::Three => 10,
        card::CardNumber::King => 4,
        card::CardNumber::Knight => 3,
        card::CardNumber::Knave => 2,
        _ => 0,
    }
}

pub fn rank_strength(card: card::Card) -> u8 {
    match card.value {
        card::CardNumber::Ace => 10,
        card::CardNumber::Three => 9,
        card::CardNumber::King => 8,
        card::CardNumber::Knight => 7,
        card::CardNumber::Knave => 6,
        card::CardNumber::Seven => 5,
        card::CardNumber::Six => 4,
        card::CardNumber::Five => 3,
        card::CardNumber::Four => 2,
        card::CardNumber::Two => 1,
    }
}

pub fn calculate_score(cards: &[card::Card]) -> u8 {
    cards.iter().copied().map(card_points).sum()
}

pub fn wins_first(lead: card::Card, follow: card::Card, briscola_suit: card::CardSuit) -> bool {
    if lead.suit == follow.suit {
        return rank_strength(lead) > rank_strength(follow);
    }

    if lead.suit == briscola_suit {
        return true;
    }

    follow.suit != briscola_suit
}

pub fn trick_points(lead: card::Card, follow: card::Card) -> u8 {
    card_points(lead) + card_points(follow)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(value: card::CardNumber, suit: card::CardSuit) -> card::Card {
        card::Card::new(value, suit)
    }

    #[test]
    fn trump_beats_non_trump() {
        let lead = card(card::CardNumber::Ace, card::CardSuit::Cups);
        let follow = card(card::CardNumber::Two, card::CardSuit::Swords);

        assert!(!wins_first(lead, follow, card::CardSuit::Swords));
    }

    #[test]
    fn higher_trump_beats_lower_trump() {
        let lead = card(card::CardNumber::Two, card::CardSuit::Swords);
        let follow = card(card::CardNumber::Three, card::CardSuit::Swords);

        assert!(!wins_first(lead, follow, card::CardSuit::Swords));
    }

    #[test]
    fn led_suit_beats_off_suit_without_trump() {
        let lead = card(card::CardNumber::Two, card::CardSuit::Cups);
        let follow = card(card::CardNumber::Ace, card::CardSuit::Coins);

        assert!(wins_first(lead, follow, card::CardSuit::Swords));
    }

    #[test]
    fn higher_led_suit_card_can_win() {
        let lead = card(card::CardNumber::Two, card::CardSuit::Cups);
        let follow = card(card::CardNumber::Three, card::CardSuit::Cups);

        assert!(!wins_first(lead, follow, card::CardSuit::Swords));
    }

    #[test]
    fn total_deck_score_is_120() {
        let score = card::Card::load_cards()
            .iter()
            .copied()
            .map(card_points)
            .sum::<u8>();

        assert_eq!(score, 120);
    }
}
