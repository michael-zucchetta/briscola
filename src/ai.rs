use crate::card;
use crate::rules;
use rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AiDifficulty {
    Random,
    Challenger,
}

impl AiDifficulty {
    pub fn label(self) -> &'static str {
        match self {
            AiDifficulty::Random => "random",
            AiDifficulty::Challenger => "challenger",
        }
    }
}

#[derive(Clone, Copy)]
pub struct VisibleGameState {
    pub briscola_suit: card::CardSuit,
    pub lead_card: Option<card::Card>,
}

pub fn choose_card(
    difficulty: AiDifficulty,
    state: VisibleGameState,
    hand: &[card::Card],
) -> usize {
    match difficulty {
        AiDifficulty::Random => random_card(hand),
        AiDifficulty::Challenger => challenger_card(state, hand),
    }
}

fn random_card(hand: &[card::Card]) -> usize {
    rand::thread_rng().gen_range(0..hand.len())
}

fn challenger_card(state: VisibleGameState, hand: &[card::Card]) -> usize {
    match state.lead_card {
        Some(lead) => choose_follow_card(state, hand, lead),
        None => choose_lead_card(state, hand),
    }
}

fn choose_lead_card(state: VisibleGameState, hand: &[card::Card]) -> usize {
    hand.iter()
        .enumerate()
        .min_by_key(|(_, card)| lead_cost(**card, state.briscola_suit))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn choose_follow_card(state: VisibleGameState, hand: &[card::Card], lead: card::Card) -> usize {
    let winning_non_trump = hand
        .iter()
        .enumerate()
        .filter(|(_, card)| {
            card.suit != state.briscola_suit
                && !rules::wins_first(lead, **card, state.briscola_suit)
        })
        .min_by_key(|(_, card)| follow_cost(**card, state.briscola_suit));

    if rules::card_points(lead) > 0 {
        if let Some((index, _)) = winning_non_trump {
            return index;
        }

        // Save trump for the deck's biggest point cards: the Three (10) and
        // Ace (11). Lower-value court cards are not worth spending a trump on.
        if rules::card_points(lead) >= 10 {
            if let Some((index, _)) = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| {
                    card.suit == state.briscola_suit
                        && !rules::wins_first(lead, **card, state.briscola_suit)
                })
                .min_by_key(|(_, card)| follow_cost(**card, state.briscola_suit))
            {
                return index;
            }
        }
    }

    hand.iter()
        .enumerate()
        .min_by_key(|(_, card)| follow_cost(**card, state.briscola_suit))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn lead_cost(card: card::Card, briscola_suit: card::CardSuit) -> u16 {
    let trump_penalty = if card.suit == briscola_suit { 40 } else { 0 };
    u16::from(trump_penalty + rules::card_points(card)) * 10 + u16::from(rules::rank_strength(card))
}

fn follow_cost(card: card::Card, briscola_suit: card::CardSuit) -> u16 {
    let trump_penalty = if card.suit == briscola_suit { 30 } else { 0 };
    u16::from(trump_penalty + rules::card_points(card)) * 10 + u16::from(rules::rank_strength(card))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(value: card::CardNumber, suit: card::CardSuit) -> card::Card {
        card::Card::new(value, suit)
    }

    #[test]
    fn challenger_wins_point_card_when_following() {
        let state = VisibleGameState {
            briscola_suit: card::CardSuit::Swords,
            lead_card: Some(card(card::CardNumber::Three, card::CardSuit::Cups)),
        };
        let hand = [
            card(card::CardNumber::Two, card::CardSuit::Coins),
            card(card::CardNumber::Four, card::CardSuit::Swords),
            card(card::CardNumber::Ace, card::CardSuit::Coins),
        ];

        assert_eq!(choose_card(AiDifficulty::Challenger, state, &hand), 1);
    }

    #[test]
    fn challenger_leads_low_non_trump() {
        let state = VisibleGameState {
            briscola_suit: card::CardSuit::Swords,
            lead_card: None,
        };
        let hand = [
            card(card::CardNumber::Ace, card::CardSuit::Coins),
            card(card::CardNumber::Two, card::CardSuit::Cups),
            card(card::CardNumber::Four, card::CardSuit::Swords),
        ];

        assert_eq!(choose_card(AiDifficulty::Challenger, state, &hand), 1);
    }
}
