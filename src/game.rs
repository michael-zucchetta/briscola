use crate::deck;
use crate::card;
use crate::constants;
use crate::game;
use crate::painter;
use crate::player;

extern crate rand;

// use rand::rngs::{OsRng, RngCore};

pub enum PlayersSize {
    Two,
    // Four,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PlayerTurn {
    Player1,
    Player2,
    // Player3,
    // Player4,
}

pub struct Game<T: painter::Painter, Y: game::UserInput> {
    deck: deck::Deck,
    briscola: card::Card,
    players: [player::Player<Y>; 2],
    players_size: PlayersSize,
    player_turn: PlayerTurn,
    painter: T
}

impl<T, Y> Game<T, Y> where T: painter::Painter, Y: game::UserInput {
    fn winner_from_scores(score1: u8, score2: u8) -> usize {
        if score1 > score2 {
            1
        } else {
            2
        }
    }

    pub fn new(
        players_size: PlayersSize,
        player_types: [player::PlayerType; 2],
        painter: T,
        user_input: Y
    ) -> Game<T, Y> {
        let deck = deck::Deck::new();
        // TODO: change to have multiple players (two or four)
        let players = match players_size {
            PlayersSize::Two => [
                player::Player::new(player_types[0], user_input.clone()),
                player::Player::new(player_types[1], user_input)
            ]
            //PlayersSize::Four =>
        };
        let value = rand::random_range(0..2);
        let turn = match value {
            0 => PlayerTurn::Player1,
            1 => PlayerTurn::Player2,
            _ => panic!("value generated outside of range"),
        };
        let briscola = deck.get_briscola();
        println!("Coin toss selected {:?}", turn);
        Game {
            deck: deck,
            briscola: briscola,
            players: players,
            players_size: players_size,
            player_turn: turn,
            painter: painter
        }
    }

    fn assign_cards(&self, initial: bool) {
        // constants.HAND_SIZE
        for player in self.players.iter() {
            if initial {
              let mut hand = Vec::with_capacity(3);
              for _ in 0..constants::HAND_SIZE {
                hand.push(self.deck.get_card().unwrap());
              }
              player.assign_cards(hand);
            } else {
              player.add_card_to_hand(self.deck.get_card().unwrap());
            }
        }
    }

    pub fn wins_first(&self, card1: card::Card, card2: card::Card) -> bool {
        if card1.suit == self.briscola.suit {
           if card2.suit != self.briscola.suit {
              true
           } else {
             card1.value.eval() > card2.value.eval()
           }
        } else {
          if card2.suit == self.briscola.suit {
             false
          } else {
             card1.value.eval() > card2.value.eval()
          }
        }
    }

    pub fn turn(&mut self, initial: bool) {
        self.assign_cards(initial);
        let last_turn = self.deck.is_deck_empty();
        let plays_size = if last_turn {
          3usize
        } else {
          1usize
        };
        for i in 0..plays_size {
            println!("N. {}", i);
            let ((player1, player1_index), (player2, player2_index)) =
                if self.player_turn == PlayerTurn::Player1 {
                (
                    (self.players.get(0).unwrap(), 1usize),
                    (self.players.get(1).unwrap(), 2usize),
                )
            } else {
                (
                    (self.players.get(1).unwrap(), 2usize),
                    (self.players.get(0).unwrap(), 1usize),
                )
            };
            let card1 = player1.play_card(player1_index);
            let card2 = player2.play_card(player2_index);
            println!(
                "Card played by {} {} and card played by {} {}",
                player1_index, card1, player2_index, card2
            );
            let mut cards_won = Vec::with_capacity(2);
            cards_won.push(card1);
            cards_won.push(card2);
            if self.wins_first(card1, card2) {
               println!("Player {} won turn", player1_index);
               player1.add_won_cards(&mut cards_won);
               self.player_turn = if player1_index == 1 {
                   PlayerTurn::Player1
               } else {
                   PlayerTurn::Player2
               };
            } else {
               println!("Player {} won turn", player2_index);
               player2.add_won_cards(&mut cards_won);
               self.player_turn = if player2_index == 1 {
                   PlayerTurn::Player1
               } else {
                   PlayerTurn::Player2
               };
            }
        }
    }

    pub fn game(&mut self) -> usize {
        println!("Beginning game");
        let mut i = 0u8;
        while !self.deck.is_deck_empty() {
          i = i + 1;
          println!("Turn {}", i);
          self.turn(i == 1);
        }
        let score1 = self.players.get(0).unwrap().calculate_score();
        let score2 = self.players.get(1).unwrap().calculate_score();
        println!("Player 1 score is {}", score1);
        println!("Player 2 score is {}", score2);
        Self::winner_from_scores(score1, score2)
    }
}

pub trait UserInput: Send + Clone {
    fn user_input(&self, player_index: usize, hand: &[card::Card]) -> usize;
}

#[cfg(test)]
mod tests {
    use crate::console;
    use crate::game::*;
    use crate::player;

    #[test]
    fn winner_from_scores_returns_player_one() {
        let winner = Game::<console::ConsolePainter, console::Console>::winner_from_scores(70, 50);
        assert_eq!(winner, 1);
    }

    #[test]
    fn winner_from_scores_returns_player_two() {
        let winner = Game::<console::ConsolePainter, console::Console>::winner_from_scores(50, 70);
        assert_eq!(winner, 2);
    }

    #[test]
    fn winner_from_scores_never_returns_zero() {
        let winner = Game::<console::ConsolePainter, console::Console>::winner_from_scores(70, 50);
        assert_ne!(winner, 0);
    }

    #[test]
    fn coin_toss_only_uses_known_turns() {
        let game = Game::new(
            PlayersSize::Two,
            [player::PlayerType::AI, player::PlayerType::AI],
            console::ConsolePainter::new(),
            console::Console::new([player::PlayerType::AI, player::PlayerType::AI]),
        );

        assert!(matches!(game.player_turn, PlayerTurn::Player1 | PlayerTurn::Player2));
    }
}
