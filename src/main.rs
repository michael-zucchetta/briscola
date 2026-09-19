use briscola::console;
use briscola::game;
use briscola::player;
use std::env;

fn read_player_type(flag_name: &str, value: Option<String>) -> player::PlayerType {
    let value = value.unwrap_or_else(|| {
        eprintln!("Missing value for {}", flag_name);
        print_usage_and_exit();
    });

    player::PlayerType::from_cli(&value).unwrap_or_else(|| {
        eprintln!(
            "Invalid value '{}' for {}. Expected 'human' or 'ai'.",
            value, flag_name
        );
        print_usage_and_exit();
    })
}

fn print_usage_and_exit() -> ! {
    eprintln!("Usage: briscola --player1 <human|ai> --player2 <human|ai>");
    std::process::exit(1);
}

fn main() {
    let mut args = env::args().skip(1);
    let mut player1 = player::PlayerType::AI;
    let mut player2 = player::PlayerType::AI;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--player1" => player1 = read_player_type("--player1", args.next()),
            "--player2" => player2 = read_player_type("--player2", args.next()),
            "--help" | "-h" => print_usage_and_exit(),
            _ => {
                eprintln!("Unknown argument '{}'", arg);
                print_usage_and_exit();
            }
        }
    }

    println!("Player 1 mode: {:?}", player1);
    println!("Player 2 mode: {:?}", player2);

    let mut game = game::Game::new(
        game::PlayersSize::Two,
        [player1, player2],
        console::ConsolePainter::new(),
        console::Console::new([player1, player2]),
    );
    let player_won = game.game();
    match player_won {
        Some(player) => println!("Player {} won", player),
        None => println!("Draw"),
    }
}
