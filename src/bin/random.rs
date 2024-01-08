use std::error::Error;

use herb::drmecref::DrMecRef;
use herb::othello::Game;
use herb::othello::Move::Pass;
use herb::{GameInterface, Player};

/// Plays through a game of Othello interfacing with Dr. Cameron's referee.
/// Makes random moves from the list of legal moves on each turn.
fn main() -> Result<(), Box<dyn Error>> {
    // setup the game
    let mut game: Game = Game::new();
    let mut opponent = DrMecRef::new();
    let herb_color = opponent.init()?;
    // Let the ref know we are ready
    opponent.ready(herb_color);

    // Game loop
    loop {
        // println!("Board:\n{}\n", game.get_board()); // debug, violates the referee

        let legal_moves = game.legal_moves();
        if game.to_move() == herb_color {
            let mv = if !legal_moves.is_empty() {
                legal_moves[rand::random::<usize>() % legal_moves.len()]
            } else {
                Pass
            };
            opponent.send_move(mv, herb_color)?;
            game.play_next_turn(mv)?;
        } else {
            // it the opponents turn, get their next move and update the game
            let opponents_move = opponent.get_next_move(game);
            game.play_next_turn(opponents_move)?;
        }
    } // end game loop
}
