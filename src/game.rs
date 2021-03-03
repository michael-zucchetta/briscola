use crate::deck;
use crate::card;
use crate::game;
use crate::painter;
use crate::player;

extern crate rand;

use rand::rngs::EntropyRng;


pub enum GameMode {
    AIvsAI,
    PlayerVsPlayer,
    PlayerVsAI,
}

pub enum PlayersSize {
    Two,
    // Four,
}

pub enum PlayerTurn {
    Player1,
    Player2,
    // Player3,
    // Player4,
}

pub struct Game<T: painter::Painter, Y: game::UserInput> {
    deck: deck::Deck,
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
                    GameMode::AIvsAI => {
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
        let mut rng = EntropyRng::new();
        let value = rng.next_usize() % 2;
        let turn = match value {
            0 => PlayerTurn::Player1,
            1 => PlayerTurn::Player2,
        };
        Game {
            deck: deck,
            players: players,
            game_mode: game_mode,
            players_size: players_size,
            player_turn: turn,
            painter: painter
        }
    }


    pub fn turn() {

    }
}

pub trait UserInput: Send + Clone {
    fn user_input() -> usize; 
}
