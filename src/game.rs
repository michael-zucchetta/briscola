use crate::deck;
use crate::card;
use crate::player;

pub enum GameMode {
    AIvsAI,
    PlayerVsPlayer,
    PlayerVsAI,
}

pub enum PlayersSize {
    Two,
    // Four,
}

pub struct Game {
    deck: deck::Deck,
    players: [player::Player; 2],
    game_mode: GameMode,
    players_size: PlayersSize,
}

impl Game {
    pub fn new(players_size: PlayersSize, game_mode: GameMode) -> Game {
        let deck = deck::Deck::new();
        // TODO: change to have multiple players (two or four)
        let players = match players_size {
            PlayersSize::Two =>
                match game_mode {
                    GameMode::AIvsAI => {
                        [
                            player::Player::new(player::PlayerType::AI),
                            player::Player::new(player::PlayerType::AI)
                        ]
                    },
                    GameMode::PlayerVsAI => {
                        [player::Player::new(player::PlayerType::Player),
                        player::Player::new(player::PlayerType::AI)]
                    },
                    GameMode::PlayerVsPlayer => {
                        [player::Player::new(player::PlayerType::Player),
                        player::Player::new(player::PlayerType::Player)]
                    },
                }
            //PlayersSize::Four =>
        };
        Game {
            deck: deck,
            players: players,
            game_mode: game_mode,
            players_size: players_size
        }
    }


}

pub trait LiveGame {
    fn user_input(self) -> card::Card;
}
