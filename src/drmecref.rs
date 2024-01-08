//! Interface to Dr. Cameron's Referee.
//!
//! The [`DrMecRef`] struct implements the [`Player`] and [`GameInterface`] traits.
//!
use std::fmt::Display;
use std::io;
use std::io::ErrorKind::InvalidInput;
use std::io::{stdin, Error};

use crate::othello::Color::{Black, White};
use crate::othello::{Color, Game, Move};
use crate::{GameInterface, Player};

/// Maps column indexes to their character values
fn map_col(col: u8) -> &'static str {
    match col {
        0 => "a",
        1 => "b",
        2 => "c",
        3 => "d",
        4 => "e",
        5 => "f",
        6 => "g",
        7 => "h",
        _ => "",
    }
}

/// Maps column characters back to their indexes
fn unmap_col(col: &str) -> u8 {
    match col {
        "a" => 0,
        "b" => 1,
        "c" => 2,
        "d" => 3,
        "e" => 4,
        "f" => 5,
        "g" => 6,
        "h" => 7,
        _ => 11,
    }
}

/// Interface to Dr. Cameron's Referee
pub struct DrMecRef {}

impl Default for DrMecRef {
    fn default() -> Self {
        Self::new()
    }
}

impl DrMecRef {
    pub fn new() -> Self {
        DrMecRef {}
    }

    /// Print a message in as a comment to the referee
    pub fn comment(message: impl Display) {
        println!("C {}", message);
    }

    /// Tell the referee we are ready to play as the given [`Color`].
    pub fn ready(&self, color: Color) {
        match color {
            White => {
                println!("R W");
            }
            Black => {
                println!("R B");
            }
        }
    }

    /// Read on stdin waiting to get the initialization message from the referee.
    pub fn init(&self) -> Result<Color, Error> {
        let mut input: String = String::new();
        stdin().read_line(&mut input)?;
        let input_lower: String = input.to_lowercase();

        if input_lower.starts_with('i') {
            if input_lower.contains('b') {
                Ok(Black)
            } else if input_lower.contains('w') {
                Ok(White)
            } else {
                Err(Error::from(InvalidInput))
            }
        } else {
            Err(Error::from(InvalidInput))
        }
    }
}

impl Player for DrMecRef {
    fn get_next_move(&mut self, _board: Game) -> Move {
        if let Ok(mv) = self.receive_move() {
            mv
        } else {
            Move::Pass
        }
    }
}

impl GameInterface for DrMecRef {
    fn send_move(&self, mv: Move, color: Color) -> io::Result<()> {
        match mv {
            Move::Move(_position) => {
                if let Some(col) = mv.get_col() {
                    if let Some(row) = mv.get_row() {
                        let col = map_col(col);
                        let row = row + 1;
                        println!("{} {} {}", color, col, row);
                    }
                }
                Ok(())
            }

            Move::Pass => {
                println!("{}", color);
                Ok(())
            }
        }
    }

    fn receive_move(&self) -> io::Result<Move> {
        let mut input = String::new();
        loop {
            input.clear();
            stdin().read_line(&mut input)?;
            if input.starts_with('B') || input.starts_with('W') {
                break;
            } else {
                DrMecRef::comment(&input);
            }
        }

        // parse the move
        if input.len() > 1 {
            let mut tokens = input.trim().split(' ');

            let _color = tokens
                .next()
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Missing color"))?;

            let col = tokens
                .next()
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Missing column"))?;
            let col = unmap_col(col);

            let row_str = tokens
                .next()
                .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Missing row"))?;

            let row = row_str
                .parse::<u8>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse row"))?;

            Ok(Move::from_col_row(col as u64, (row - 1) as u64).expect("Illegal Move."))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to parse move",
            ))
        }
    }
}
