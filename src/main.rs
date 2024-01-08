use std::env;
use std::error::Error;

use herb::config::Config;
use herb::drmecref::DrMecRef;
use herb::othello::Move::Pass;
use herb::othello::{Game, Move};
use herb::{GameInterface, Herb, Player};

/// Plays through a game of Othello interfacing with Dr. Cameron's referee.
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let config = if args.is_empty() {
        Config::default()
    } else {
        Config::new(&args[0])
    };

    let mut opponent = DrMecRef::new();
    let herb_color = opponent.init()?;

    // Let the ref know we are ready
    opponent.ready(herb_color);

    let mut herb = Herb::new(config);
    let mut game: Game = Game::new();

    // Game loop
    loop {
        // println!("Board:\n{}\n", game.get_board()); // debug, violates the referee
        DrMecRef::comment(format!("Main: start turn {}", game.get_turn()));

        if game.is_over() {
            DrMecRef::comment(format!("Main: game over at turn {}", game.get_turn()));
            break;
        }

        let legal_moves = game.legal_moves();
        if game.to_move() == herb_color {
            // it is herb's turn
            let mut herbs_move = herb.get_next_move(game);
            if !legal_moves.is_empty() && !legal_moves.contains(&herbs_move) {
                DrMecRef::comment("Main: Got an illegal move from Herb.");
                herbs_move = *legal_moves.first().unwrap_or(&Pass);
            }
            opponent.send_move(herbs_move, herb_color)?;
            game.play_next_turn(herbs_move)?;
        } else {
            // it the opponents turn, get their next move and update the game
            let opponents_move = opponent.get_next_move(game);
            match opponents_move {
                Move::Move(_mv) => {
                    DrMecRef::comment(format!("Main: got opponent move {}", opponents_move))
                }
                Pass => {
                    DrMecRef::comment("Main: got opponent move Pass");
                }
            }
            game.play_next_turn(opponents_move)?;
        }
    } // end game loop

    Ok(())
}
