use crate::deck;
use crate::card;
use crate::hand;
use crate::constants;
use crate::game;
use crate::painter;
use crate::player;

extern crate rand;
extern crate rand_core;

use rand::RngCore;
use rand_core::OsRng;

// use rand::rngs::{OsRng, RngCore};


trait Display {}

pub enum GameMode {
    AIVsAI,
    PlayerVsPlayer,
    PlayerVsAI,
}

pub enum PlayersSize {
    Two,
    // Four,
}

#[derive(PartialEq, Clone, Copy)]
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
    game_mode: GameMode,
    players_size: PlayersSize,
    player_turn: PlayerTurn,
    painter: T
}

impl<T, Y> Game<T, Y> where T: painter::Painter, Y: game::UserInput {

    pub fn new(players_size: PlayersSize, game_mode: GameMode, painter: T, user_input: Y) -> Game<T, Y> {
        let deck = deck::Deck::new();
        // TODO: change to have multiple players (two or four)
        let players = match players_size {
            PlayersSize::Two =>
                match game_mode {
                    GameMode::AIVsAI => {
                        [
                            player::Player::new(player::PlayerType::AI, user_input.clone()),
                            player::Player::new(player::PlayerType::AI, user_input)
                        ]
                    },
                    GameMode::PlayerVsAI => {
                        [player::Player::new(player::PlayerType::Player, user_input.clone()),
                        player::Player::new(player::PlayerType::AI, user_input)]
                    },
                    GameMode::PlayerVsPlayer => {
                        [player::Player::new(player::PlayerType::Player, user_input.clone()),
                        player::Player::new(player::PlayerType::Player, user_input)]
                    },
                }
            //PlayersSize::Four =>
        };
        // let mut key = [0u8; 16];
        let value = OsRng.next_u32() as usize % 2;
        let turn = match value {
            0 => PlayerTurn::Player1,
            1 => PlayerTurn::Player2,
            _ => panic!("value generated outside of range"),
        };
        let briscola = deck.get_briscola();
        Game {
            deck: deck,
            briscola: briscola,
            players: players,
            game_mode: game_mode,
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

    pub fn turn(&self, initial: bool) {
        self.assign_cards(initial);
        let last_turn = self.deck.is_deck_empty();
        let plays_size = if last_turn {
          3usize
        } else {
          1usize
        };
        for i in 0..plays_size {
            let (player1, player2) = if self.player_turn == PlayerTurn::Player1 {
                (
                    self.players.get(0).unwrap(),
                    self.players.get(1).unwrap(),
                )
            } else {
                (
                    self.players.get(1).unwrap(),
                    self.players.get(0).unwrap(),
                )
            };
            let card1 = player1.play_card();
            T::player_played_card(card1, 1usize);
            let card2 = player2.play_card();
            T::player_played_card(card2, 2usize);
            let mut cards_won = Vec::with_capacity(2);
            cards_won.push(card1);
            cards_won.push(card2);
            if self.wins_first(card1, card2) {
               T::player_won(1usize, &cards_won);
               player1.add_won_cards(&mut cards_won);
            } else {
               T::player_won(2usize, &cards_won);
               player2.add_won_cards(&mut cards_won);
            }
        }
    }

    pub fn game(&self) -> usize {
        println!("Beginning game");
        self.painter.draw_beginning(&self.deck);//self.painter);
        let mut i = 0u8;
        while !self.deck.is_deck_empty() {
          i = i + 1;
          println!("Turn {}", i);
          self.turn(i == 1);
        }
        let score1 = self.players.get(0).unwrap().calculate_score();
        let score2 = self.players.get(1).unwrap().calculate_score();
        T::player_scores(score1, score2);
        if score1 > score2 {
          0
        } else {
          1
        }
    }
}

pub trait UserInput: Clone {
    fn user_input(&self, hand: &hand::Hand) -> usize;
}
