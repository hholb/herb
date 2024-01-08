use herb::config::Config;
use herb::othello::Color::{Black, White};
use herb::othello::Game;
use herb::othello::Move::Pass;
use herb::{Herb, Player};
use rand::random;
use std::env;
use std::time::Instant;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let config = if args.is_empty() {
        Config::default()
    } else {
        Config::new(&args[0])
    };
    let mut game = Game::new();

    let mut black = Herb::new(config);

    let start_time = Instant::now();
    while !game.is_over() {
        println!("\nTurn {}: \n{}", game.get_turn(), game.get_board());
        let next_move = match game.to_move() {
            White => {
                let legal_moves = game.legal_moves();
                if !legal_moves.is_empty() {
                    legal_moves[random::<usize>() % legal_moves.len()]
                } else {
                    Pass
                }
            }
            Black => black.get_next_move(game),
        };
        game.play_next_turn(next_move)?;
    }
    let end_time = Instant::now();
    println!("Game Over after {} turns.", game.get_turn());
    println!("Duration: {}ms", (end_time - start_time).as_millis());
    println!("Score: {}", game.score());
    Ok(())
}
