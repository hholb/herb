//! # Othello engine
//!
//! This module contains the logic and data structures for a game of Othello.
//!
//! # Rules
//! An 8x8 board is used with discs that are black on one side and white on the other.
//! Two players take turns placing discs on the board with their assigned color facing up.
//! The objective is to end the game with more discs of your color than your opponent.
//! The game ends when neither player can place a disc on the board.
//!
//! Legal moves are those where at least one of the opponent's discs is flanked by the
//! new disc. This means the new disc must 'capture' at least one of the opponent's discs
//! by surrounding it on two opposite sides. All of the opponent's discs between the new
//! disc and another of the player's discs are flipped to the player's color.
//!
//! The game starts with two black discs on d5 and e4, and two white discs on d4 and e5.
//! The player with the black discs moves first.
//!
//! The player with the most discs on the board at the end of the game is the winner.
//!
//! # Bitboard representation
//!
//! ```rust
//! struct Bitboard {
//!     black: u64,
//!     white: u64,
//! }
//! ```
//!
//! The board is represented by two 64-bit unsigned-integers, one for each color. Each bit represents
//! a square on the board. The least significant bit represents the top left square, and the
//! most significant bit represents the bottom right square. If a bit is set to 1, it means
//! that square is occupied by a disc of that color. If a bit is set to 0, it means that
//! square is empty.
//!
//!```text
//!     0    1    2    3    4    5    6    7
//!     A    B    C    D    E    F    G    H
//!      +--------------------------------+
//! 0 | 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 |
//!      +--------------------------------+
//! 1 | 08 | 09 | 10 | 11 | 12 | 13 | 14 | 15 |
//!      +--------------------------------+
//! 2 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 |
//!      +--------------------------------+
//! 3 | 24 | 25 | 26 | 27 | 28 | 29 | 30 | 31 |
//!      +--------------------------------+
//! 4 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 |
//!      +--------------------------------+
//! 5 | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 |
//!      +--------------------------------+
//! 6 | 48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 |
//!      +--------------------------------+
//! 7 | 56 | 57 | 58 | 59 | 60 | 61 | 62 | 63 |
//!      +--------------------------------+
//! ```
//!
//! # Initial board state
//!
//! Black pieces on d5 and e4 with bit values: `let black_starting_positions: u64 = 1<<28 | 1<<35;`
//!
//! White piece on d4 and e5 with bit values: `let white_starting_positions: u64 = 1<<27 | 1<<36;`
//!
//! ```text
//!     0    1    2    3    4    5    6    7
//!     A    B    C    D    E    F    G    H
//!      +--------------------------------+
//! 0 | 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 |
//!      +--------------------------------+
//! 1 | 08 | 09 | 10 | 11 | 12 | 13 | 14 | 15 |
//!      +--------------------------------+
//! 2 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 |
//!      +--------------------------------+
//! 3 | 24 | 25 | 26 | W  | B  | 29 | 30 | 31 |
//!      +--------------------------------+
//! 4 | 32 | 33 | 34 | B  | W  | 37 | 38 | 39 |
//!      +--------------------------------+
//! 5 | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 |
//!      +--------------------------------+
//! 6 | 48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 |
//!      +--------------------------------+
//! 7 | 56 | 57 | 58 | 59 | 60 | 61 | 62 | 63 |
//!      +--------------------------------+
//! ```
//!
//! # Game struct
//!
//! A game of Othello is represented by the [`Game`] struct which
//! keeps track of whose turn it is and holds a [`Bitboard`],
//! The [`Game`] struct also provides functions for advancing the game and getting a list of legal moves.
//! The game is started by creating a new [`Game`] instance:
//!```rust
//! use herb::othello::Game;
//! let game: Game = Game::new();
//! ```
//!
//! The game progresses by calling the `play_next_turn()` function on the [`Game`]
//! and passing in an optional [`Move`] struct. If `mv` is [`None`] then the current player
//! passes and the game updates the turn count and switches the current player without updating
//! the board.
//!```rust
//! use herb::othello::{Game, Move};
//! use herb::othello::Color::{Black, White};
//!
//! let mut game: Game = Game::new(); // game starts with 4 pieces on the board, black to move
//! let current_player = game.to_move();
//! assert_eq!(current_player, Black);
//!
//! let black_pieces = game.get_board().get_black();
//! let white_pieces = game.get_board().get_white();
//! assert_eq!(black_pieces.count_ones(), 2);
//! assert_eq!(white_pieces.count_ones(), 2);
//!
//! let legal_moves: Vec<Move> = game.legal_moves();
//! if !legal_moves.is_empty() {
//!    let mv: Move = legal_moves[0];
//!    game.play_next_turn(mv).unwrap(); // board gets updated, player switches
//!
//!    let current_player = game.to_move();
//!    assert_eq!(current_player, White);
//!
//!    let black_pieces = game.get_board().get_black();
//!    let white_pieces = game.get_board().get_white();
//!    assert_eq!(black_pieces.count_ones(), 4);
//!    assert_eq!(white_pieces.count_ones(), 1);
//! }
//! ```
//!
//! # Example: Simple Game
//!
//! This simple loop should play through a legal game of Othello
//! until neither player can make a legal [`Move`].
//!
//!```rust
//! use herb::othello::{Move, Game};
//!
//! let mut game = Game::new();
//!
//! while !game.is_over() {
//!    let legal_moves = game.legal_moves();
//!
//!    if !legal_moves.is_empty() {
//!        let next_move = legal_moves[0];
//!        // Some(&Move) plays a move for the current player.
//!        game.play_next_turn(next_move).unwrap();
//!    } else {
//!        // passing None counts as a passed move and play moves to the next player
//!        game.play_next_turn(Move::Pass).unwrap();
//!    }
//! }
//!
//! println!("Game ended at turn: {}", game.get_turn());
//! ```
//!

use rand::Rng;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::othello::Color::{Black, White};
use crate::othello::GameError::{GameOver, InvalidMove};
use crate::othello::SearchDirection::{
    DiagonalDownLeft, DiagonalDownRight, DiagonalUpLeft, DiagonalUpRight, Down, Left, Right, Up,
};

use crate::othello::Move::Pass;
use serde::{Deserialize, Serialize};

const BLACK_INITIAL_POSITIONS: u64 = 1 << 28 | 1 << 35;
const WHITE_INITIAL_POSITIONS: u64 = 1 << 27 | 1 << 36;

const LEFT_EDGE_MASK: u64 = left_edge_mask();
const RIGHT_EDGE_MASK: u64 = right_edge_mask();

// Directions in this order: Up, Left, Diagonal Up-Left, Diagonal Down-Left,
// Down, Right, Diagonal Down-Right, Diagonal Up-Right
const DIRECTION_MASKS: [u64; 8] = [
    0x7F7F7F7F7F7F7F7F,
    0x00FFFFFFFFFFFFFF,
    0x007F7F7F7F7F7F7F,
    0x00FEFEFEFEFEFEFE,
    0xFEFEFEFEFEFEFEFE,
    0xFFFFFFFFFFFFFF00,
    0xFEFEFEFEFEFEFE00,
    0x7F7F7F7F7F7F7F00,
];

const DIRECTION_OFFSETS: [u64; 4] = [1, 8, 9, 7];

pub const CORNERS: [Move; 4] = [Move::Move(0), Move::Move(7), Move::Move(56), Move::Move(63)];

pub const X_MOVES: [Move; 12] = [
    Move::Move(1),
    Move::Move(6),
    Move::Move(8),
    Move::Move(9),
    Move::Move(14),
    Move::Move(15),
    Move::Move(48),
    Move::Move(49),
    Move::Move(54),
    Move::Move(55),
    Move::Move(57),
    Move::Move(62),
];

pub const EDGES: [Move; 28] = [
    Move::Move(0),
    Move::Move(1),
    Move::Move(2),
    Move::Move(3),
    Move::Move(4),
    Move::Move(5),
    Move::Move(6),
    Move::Move(7),
    Move::Move(8),
    Move::Move(16),
    Move::Move(24),
    Move::Move(32),
    Move::Move(40),
    Move::Move(48),
    Move::Move(56),
    Move::Move(15),
    Move::Move(23),
    Move::Move(31),
    Move::Move(39),
    Move::Move(47),
    Move::Move(55),
    Move::Move(57),
    Move::Move(58),
    Move::Move(59),
    Move::Move(60),
    Move::Move(61),
    Move::Move(62),
    Move::Move(63),
];

const DIAGONALS: [Move; 16] = [
    Move::Move(0),
    Move::Move(9),
    Move::Move(18),
    Move::Move(27),
    Move::Move(36),
    Move::Move(45),
    Move::Move(54),
    Move::Move(63),
    Move::Move(7),
    Move::Move(14),
    Move::Move(21),
    Move::Move(28),
    Move::Move(35),
    Move::Move(42),
    Move::Move(49),
    Move::Move(56),
];

const CENTER_4: [Move; 4] = [
    Move::Move(27),
    Move::Move(28),
    Move::Move(35),
    Move::Move(36),
];

const INNER_BOARD: [Move; 16] = [
    Move::Move(18),
    Move::Move(19),
    Move::Move(20),
    Move::Move(21),
    Move::Move(26),
    Move::Move(27),
    Move::Move(28),
    Move::Move(29),
    Move::Move(34),
    Move::Move(35),
    Move::Move(36),
    Move::Move(37),
    Move::Move(42),
    Move::Move(43),
    Move::Move(44),
    Move::Move(45),
];

/// Builds a mask with bits set along the 'left' edge of the board.
const fn left_edge_mask() -> u64 {
    let mut result: u64 = 0;
    let mut i: u8 = 0;
    while i < 64 {
        result |= 1 << i;
        i += 8;
    }
    result
}

/// Builds a mask with bits set along the 'right' edge of the board
const fn right_edge_mask() -> u64 {
    let mut result: u64 = 0;
    let mut i: u8 = 7;
    while i < 64 {
        result |= 1 << i;
        i += 8;
    }
    result
}

/// Holds the state of a game of Othello.
///
/// Some functions update the state and require Game variables
/// to be declared as mut: `let mut game = Game::new();`.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Game {
    current_board: Bitboard,
    current_player: Color,
    turn: i32,
}

impl Game {
    /// Creates new Game state initializes the board with the pieces
    /// in their starting positions and sets the current player to black.
    pub fn new() -> Self {
        Game {
            turn: 0,
            current_player: Black,
            current_board: Bitboard::new(),
        }
    }

    /// Progresses the game by one turn. If given a valid move the board will be updated
    /// with the new piece and all appropriate opponent pieces flipped. If mv is [`Pass`],
    /// it is taken as a pass by the current player and the game moves forward one turn
    /// switching to the other player, but it does not update the board.
    ///
    /// If the given move is illegal by the rules of Othello, an [`InvalidMove`] error is returned
    /// and the game state is unchanged.
    pub fn play_next_turn(&mut self, mv: Move) -> Result<(), GameError> {
        if self.is_over() {
            return Err(GameOver);
        }

        let legal_moves = self.legal_moves();

        if !legal_moves.is_empty() {
            match mv {
                Move::Move(_position) => {
                    if legal_moves.contains(&mv) {
                        self.apply_move(mv);
                    } else {
                        return Err(InvalidMove);
                    }
                }
                Move::Pass => self.apply_move(mv),
            }
        }

        self.turn += 1;

        // switch players
        self.current_player = match self.current_player {
            Black => White,
            White => Black,
        };

        Ok(())
    }

    /// Applies the given move to the internal board and flips appropriate pieces.
    /// DOES NOT CHECK FOR INVALID MOVES.
    /// If given an invalid move the behavior is undefined.
    ///
    /// Returns true if update was successful. It is all bit-wise operations from here,
    /// so if something fails there is a bigger problem.
    fn apply_move(&mut self, mv: Move) {
        let player = self.current_player;

        // make copies of the board values to work on
        let (mut new_black, mut new_white) = (self.current_board.black, self.current_board.white);

        if let Some(position) = mv.get_position() {
            // depending on whose turn it is, set the position bit and flip the appropriate bits
            match player {
                Black => {
                    new_black |= position;
                    let directions = [
                        Left,
                        Right,
                        Down,
                        Up,
                        DiagonalUpRight,
                        DiagonalDownRight,
                        DiagonalUpLeft,
                        DiagonalDownLeft,
                    ];
                    // look in each direction for pieces that would be flipped
                    for &dir in directions.iter() {
                        self.flip(position, &mut new_black, &mut new_white, dir);
                    }
                }
                White => {
                    new_white |= position;
                    let directions = [
                        Left,
                        Right,
                        Down,
                        Up,
                        DiagonalUpRight,
                        DiagonalDownRight,
                        DiagonalUpLeft,
                        DiagonalDownLeft,
                    ];
                    // look in each direction for pieces that would be flipped
                    for &dir in directions.iter() {
                        self.flip(position, &mut new_white, &mut new_black, dir);
                    }
                }
            }

            // replace the current_board values with the updated copies
            self.current_board.black = new_black;
            self.current_board.white = new_white;
        }
    }

    /// Returns a vector of all legal moves for the current player.
    ///
    /// If the returned vector is empty, there are no legal moves for the
    /// current player.
    pub fn legal_moves(&self) -> Vec<Move> {
        let mut legal_moves = Vec::new();
        let (player_pieces, opponent_pieces) = match self.current_player {
            Black => (self.current_board.black, self.current_board.white),
            White => (self.current_board.white, self.current_board.black),
        };

        let empty_tiles = !(self.current_board.black | self.current_board.white);
        let mut valid_moves: u64 = 0;

        for i in 0..4 {
            let mut neighbors =
                ((player_pieces & DIRECTION_MASKS[i]) << DIRECTION_OFFSETS[i]) & opponent_pieces;
            while neighbors != 0 {
                let potential_flips = (neighbors & DIRECTION_MASKS[i]) << DIRECTION_OFFSETS[i];
                valid_moves |= potential_flips & empty_tiles;
                neighbors = potential_flips & opponent_pieces;
            }

            neighbors = ((player_pieces & DIRECTION_MASKS[i + 4]) >> DIRECTION_OFFSETS[i])
                & opponent_pieces;
            while neighbors != 0 {
                let potential_flips = (neighbors & DIRECTION_MASKS[i + 4]) >> DIRECTION_OFFSETS[i];
                valid_moves |= potential_flips & empty_tiles;
                neighbors = potential_flips & opponent_pieces;
            }
        }

        // Convert the bitboard of valid moves into a vector of Move
        for row in 0..8 {
            for col in 0..8 {
                let shift = (row * 8) + col;
                let position = 1u64 << shift;
                if (valid_moves & position) != 0 {
                    // This position is a valid move, so we add it to the list of legal moves.
                    if let Ok(mv) = Move::new(position) {
                        legal_moves.push(mv);
                    }
                }
            }
        }

        legal_moves
    }

    /// Flips the opponent pieces that are captured between pos in the given direction.
    /// Updates the internal state of the game.
    fn flip(&mut self, pos: u64, own: &mut u64, opponent: &mut u64, direction: SearchDirection) {
        let mut flip = 0u64;
        let mut mask = pos;

        loop {
            // move in the given direction until we hit an edge by shifting the mask one bit at a time
            mask = match direction {
                Right => {
                    if mask & RIGHT_EDGE_MASK != 0 {
                        break;
                    }
                    mask << 1
                }
                Left => {
                    if mask & LEFT_EDGE_MASK != 0 {
                        break;
                    }
                    mask >> 1
                }
                Down => mask << 8,
                Up => mask >> 8,
                DiagonalUpRight => {
                    if mask & RIGHT_EDGE_MASK != 0 {
                        break;
                    }
                    mask << 9
                }
                DiagonalDownRight => {
                    if mask & LEFT_EDGE_MASK != 0 {
                        break;
                    }
                    mask >> 9
                }
                DiagonalDownLeft => {
                    if mask & LEFT_EDGE_MASK != 0 {
                        break;
                    }
                    mask << 7
                }
                DiagonalUpLeft => {
                    if mask & RIGHT_EDGE_MASK != 0 {
                        break;
                    }
                    mask >> 7
                }
            };

            // look if there is an opponent's piece under the mask
            if mask & *opponent != 0 {
                // there is a piece to flip
                flip |= mask;
            } else if mask & *own != 0 {
                // we hit our own piece, set the appropriate bits
                *own ^= flip;
                *opponent ^= flip;
                break;
            } else {
                break;
            }
        }
    }

    /// Returns the internal [`Bitboard`].
    pub fn get_board(&self) -> Bitboard {
        self.current_board
    }

    /// Returns the current player, as in the player who is next to make a move.
    pub fn to_move(&self) -> Color {
        self.current_player
    }

    /// Returns the current turn number
    pub fn get_turn(&self) -> usize {
        self.turn as usize
    }

    /// Returns true if there are no legal moves left for either player.
    pub fn is_over(&self) -> bool {
        // Check if the current player has legal moves
        let current_legal_moves = self.legal_moves();

        // If the current player has legal moves, the game isn't over
        if !current_legal_moves.is_empty() {
            return false;
        }

        // Simulate the next player's turn to check for legal moves
        let mut sim_game = *self;
        sim_game.turn += 1;
        sim_game.current_player = match self.current_player {
            Black => White,
            White => Black,
        };
        let next_player_legal_moves = sim_game.legal_moves();

        // If neither player has legal moves, the game is over
        current_legal_moves.is_empty() && next_player_legal_moves.is_empty()
    }

    /// Returns true if the next call to [`play_next_move`] will end the game.
    pub fn is_terminal(&self) -> bool {
        let mut sim_game = *self;
        if !sim_game.is_over() {
            let legal_moves = sim_game.legal_moves();
            if let Some(next_move) = legal_moves.first() {
                sim_game.play_next_turn(*next_move).unwrap();
                if sim_game.is_over() {
                    return true;
                }
            }
        }
        false
    }

    /// Returns the current score of the game.
    /// - `0` means currently tied
    /// -  greater than `0` means black is ahead by the amount returned
    /// -  less than `0` means white is ahead by the amount returned
    pub fn score(&self) -> i32 {
        self.current_board.black.count_ones() as i32 - self.current_board.white.count_ones() as i32
    }

    /// Returns the player that is currently winning.
    /// - `Some(Black)` Black is winning
    /// - `None` It is tied.
    /// - `Some(White)` White is winning.
    pub fn winner(&self) -> Option<Color> {
        let score = self.score();
        match score {
            score if score > 0 => Some(Black),
            0 => None,
            _ => Some(White),
        }
    }

    /// Returns the current number of empty squares on on the current board.
    pub fn empty_squares(&self) -> u64 {
        (self.current_board.black | self.current_board.white).count_zeros() as u64
    }

    /// Return a random move from the list of legal moves available to the current player.
    pub fn random_move(&self) -> Move {
        let mut rng = rand::thread_rng();
        let legal_moves = self.legal_moves();
        if !legal_moves.is_empty() {
            return legal_moves[rng.gen::<usize>() % legal_moves.len()];
        }
        Pass
    }

    /// Return the [`Move`] that leads to an opponent position with the fewest legal moves.
    /// The returned move will be from the list of legal moves available to the current player.
    pub fn move_with_lowest_opp_mobility(&self) -> Move {
        let mut lowest_mobility: usize = usize::MAX;
        let legal_moves = self.legal_moves();
        let mut best_move = Pass;
        for mv in legal_moves {
            let mut sim_game = *self;
            sim_game.play_next_turn(mv).unwrap();
            let mobility = sim_game.legal_moves().len();
            if mobility < lowest_mobility {
                lowest_mobility = mobility;
                best_move = mv;
            }
        }
        best_move
    }

    /// Return the number of legal moves available to the current player.
    pub fn mobility(&self) -> usize {
        self.legal_moves().len()
    }

    pub fn get_hash(&self) -> u64 {
        self.current_board.black | self.current_board.white
    }

    /// Return the number of corner pieces by each player.
    /// The return value is a two-tuple in the from (num_black_pieces_held, num_white_pieces_held).
    pub fn num_corners_held(&self) -> (usize, usize) {
        let mut count_black = 0;
        let mut count_white = 0;

        let black_pieces = self.current_board.black;
        let white_pieces = self.current_board.white;

        for corner in CORNERS {
            let position = corner.get_position().unwrap();
            if position & black_pieces > 0 {
                count_black += 1;
            }
            if position & white_pieces > 1 {
                count_white += 1;
            }
        }

        (count_black, count_white)
    }

    /// Return the number of pieces held by each player along the edges of the board.
    /// The return value is a two-tuple in the from (num_black_pieces_held, num_white_pieces_held).
    pub fn num_edges_held(&self) -> (usize, usize) {
        let mut count_black = 0;
        let mut count_white = 0;

        let black_pieces = self.current_board.black;
        let white_pieces = self.current_board.white;

        for edge in EDGES {
            let position = edge.get_position().unwrap();
            if position & black_pieces > 0 {
                count_black += 1;
            }
            if position & white_pieces > 1 {
                count_white += 1;
            }
        }

        (count_black, count_white)
    }

    /// Return the number of pieces held by each player in the 'x-moves' one the board. The x-moves
    /// are board positions adjacent to a corner.
    /// The return value is a two-tuple in the from (num_black_pieces_held, num_white_pieces_held).
    pub fn num_x_moves_held(&self) -> (usize, usize) {
        let mut count_black = 0;
        let mut count_white = 0;

        let black_pieces = self.current_board.black;
        let white_pieces = self.current_board.white;

        for xmove in X_MOVES {
            let position = xmove.get_position().unwrap();
            if position & black_pieces > 0 {
                count_black += 1;
            }
            if position & white_pieces > 1 {
                count_white += 1;
            }
        }

        (count_black, count_white)
    }

    /// Return the number of pieces held by each player in the 2 diagonals from corner to corner.
    /// The return value is a two-tuple in the from (num_black_pieces_held, num_white_pieces_held).
    pub fn diagonals_held(&self) -> (usize, usize) {
        let mut count_black = 0;
        let mut count_white = 0;

        let black_pieces = self.current_board.black;
        let white_pieces = self.current_board.white;

        for diagonal in DIAGONALS {
            let position = diagonal.get_position().unwrap();
            if position & black_pieces > 0 {
                count_black += 1;
            }
            if position & white_pieces > 1 {
                count_white += 1;
            }
        }

        (count_black, count_white)
    }

    /// Return the number of pieces held by each player in the center 2x2 area if the board.
    /// The return value is a two-tuple in the from (num_black_pieces_held, num_white_pieces_held).
    pub fn center_4_held(&self) -> (usize, usize) {
        let mut count_black = 0;
        let mut count_white = 0;

        let black_pieces = self.current_board.black;
        let white_pieces = self.current_board.white;

        for square in CENTER_4 {
            let position = square.get_position().unwrap();
            if position & black_pieces > 0 {
                count_black += 1;
            }
            if position & white_pieces > 1 {
                count_white += 1;
            }
        }

        (count_black, count_white)
    }

    /// Return the number of pieces held by each player in the inner 4x4 area if the board.
    /// The return value is a two-tuple in the from (num_black_pieces_held, num_white_pieces_held).
    pub fn inner_board_held(&self) -> (usize, usize) {
        let mut count_black = 0;
        let mut count_white = 0;

        let black_pieces = self.current_board.black;
        let white_pieces = self.current_board.white;

        for square in INNER_BOARD {
            let position = square.get_position().unwrap();
            if position & black_pieces > 0 {
                count_black += 1;
            }
            if position & white_pieces > 1 {
                count_white += 1;
            }
        }

        (count_black, count_white)
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

/// Holds the position on the board as a [`u64`] with a single bit set
/// in the position it would occupy in a [`Bitboard`].
#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Move {
    Move(u64),
    Pass,
}

impl Move {
    /// Creates a new Move with the given position.
    ///
    /// Returns Ok(Move) if the position is valid. `InvalidMove` otherwise.
    pub fn new(position: u64) -> Result<Self, GameError> {
        if position.count_ones() > 1 {
            Err(InvalidMove)
        } else {
            Ok(Move::Move(position))
        }
    }

    /// Creates a new Move with the given col and row values
    /// 0 indexed, col and row must be between 0 - 7 inclusive.
    ///
    /// Returns [`GameError`] if given row or column are outside of the board.
    ///
    /// Returns `Ok(Move)`
    pub fn from_col_row(col: u64, row: u64) -> Result<Self, GameError> {
        if col <= 7 && row <= 7 {
            let position: u64 = 1 << ((row * 8) + col);
            Ok(Move::Move(position))
        } else {
            Err(InvalidMove)
        }
    }

    /// Returns the column value, 0 indexed
    pub fn get_col(&self) -> Option<u8> {
        match *self {
            Move::Move(position) => Some((position.trailing_zeros() % 8) as u8),
            Move::Pass => None,
        }
    }

    /// Returns the row value
    pub fn get_row(&self) -> Option<u8> {
        match *self {
            Move::Move(position) => Some((position.trailing_zeros() / 8) as u8),
            Move::Pass => None,
        }
    }

    /// Returns the internal u64.
    pub fn get_position(&self) -> Option<u64> {
        match *self {
            Move::Move(position) => Some(position),
            Move::Pass => None,
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(col) = self.get_col() {
            if let Some(row) = self.get_row() {
                return write!(f, "{:?} {:?}", col, row);
            }
        }
        write!(f, "Invalid move")
    }
}

/// Holds two [`u64`] values representing the pieces on an 8x8 game board.
#[derive(Clone, Copy, Eq, Debug, PartialEq, Hash, Serialize, Deserialize)]
pub struct Bitboard {
    black: u64,
    white: u64,
}

impl Bitboard {
    /// Creates a new board where the initial values are set to
    /// the starting positions for a game of Othello.
    pub fn new() -> Self {
        Self {
            black: BLACK_INITIAL_POSITIONS,
            white: WHITE_INITIAL_POSITIONS,
        }
    }

    /// Returns the value of the u64 representing the black pieces
    pub fn get_black(&self) -> u64 {
        self.black
    }

    /// Returns the value of the u64 representing the white pieces
    pub fn get_white(&self) -> u64 {
        self.white
    }
}

impl Default for Bitboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        for y in 0..8 {
            for x in 0..8 {
                let pos = 1u64 << (x + y * 8);
                if self.black & pos != 0 {
                    write!(f, "B ")?;
                } else if self.white & pos != 0 {
                    write!(f, "W ")?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Enumerates the piece color options.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Color {
    White,
    Black,
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            White => write!(f, "W"),
            Black => write!(f, "B"),
        }
    }
}

/// Enumerates possible Game errors.
#[derive(Debug)]
pub enum GameError {
    InvalidMove,
    GameOver,
}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match *self {
            InvalidMove => {
                write!(f, "Invalid move!")
            }
            GameOver => {
                write!(f, "Game Over.")
            }
        }
    }
}

impl Error for GameError {}

/// Enumerates search directions for finding moves and flips.
#[derive(Clone, Copy)]
enum SearchDirection {
    Up,
    Down,
    Left,
    Right,
    DiagonalDownRight,
    DiagonalDownLeft,
    DiagonalUpLeft,
    DiagonalUpRight,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::{Hash, Hasher};

    #[test]
    fn test_initial_board_setup() {
        let game = Game::new();
        let white = game.current_board.white;
        let black = game.current_board.black;
        assert_eq!(white.count_ones(), 2);
        assert_eq!(black.count_ones(), 2);
        assert_eq!((white | black).count_ones(), 4);

        assert_eq!(white, 1 << 27 | 1 << 36);
        assert_eq!(black, 1 << 28 | 1 << 35);
    }

    #[test]
    fn test_game_over_no_legal_moves() {
        let mut game = Game::new();
        assert!(!game.is_over());

        // each board has 1 bit set, no legal moves
        let black_board: u64 = 1;
        let white_board: u64 = 1 << 2;

        game.current_board.black = black_board;
        game.current_board.white = white_board;

        assert!(game.is_over());
    }

    #[test]
    fn test_game_over_after_terminal_move() {
        let mut game = Game::new();
        // set bottom-right corner for white
        let white: u64 = 0 | (1 << 63);
        // set the whole board, unset the top-left and bottom-right corners
        let black: u64 = (!0 ^ 1) ^ (1 << 63);
        game.current_board.white = white;
        game.current_board.black = black;
        game.current_player = White;

        let only_move = Move::from_col_row(0, 0).unwrap();
        game.play_next_turn(only_move).unwrap();
        assert!(game.is_over());
    }

    #[test]
    fn test_legal_moves_initial_board() {
        let game = Game::new();
        let legal_moves = game.legal_moves();
        assert_eq!(legal_moves.len(), 4);
        assert!(legal_moves.contains(&Move::from_col_row(2, 3).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(3, 2).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(4, 5).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(5, 4).unwrap()));
    }

    #[test]
    fn test_legal_moves_terminal_move() {
        let mut game = Game::new();
        // set bottom-right corner for white
        let white: u64 = 0 | (1 << 63);
        // set the whole board, unset the top-left and bottom-right corners
        let black: u64 = (!0 ^ 1) ^ (1 << 63);
        game.current_board.white = white;
        game.current_board.black = black;
        game.current_player = White;

        let legal_moves = game.legal_moves();
        let only_move = Move::from_col_row(0, 0).unwrap();
        assert_eq!(legal_moves.len(), 1);
        assert!(legal_moves.contains(&only_move));
    }

    #[test]
    fn test_play_next_move_from_game_start() {
        let turns: usize = 10;
        let mut game = Game::new();
        for i in 1..turns {
            let legal_moves = game.legal_moves();
            game.play_next_turn(legal_moves[0]).unwrap();
            assert_eq!(game.turn, i as i32);
        }
    }

    //noinspection SpellCheckingInspection
    /// Sets up a mid-game board, validates legal moves, then plays a known move.
    /// Validates correct board state and legal moves after the known move is played.
    /// Sorry, this one is long, there are a lot of legal moves to validate.
    #[test]
    fn test_play_next_move_mid_game() {
        // ....x...
        // ..xBWx..
        // ..xWB...
        // ..BBBB..
        // ..BWWB..
        // ..BBBB..
        // ...BWx..
        // ...xxx..
        let black_positions = vec![
            (3, 1),
            (4, 2),
            (2, 3),
            (3, 3),
            (4, 3),
            (5, 3),
            (2, 4),
            (5, 4),
            (2, 5),
            (3, 5),
            (4, 5),
            (5, 5),
            (3, 6),
        ];
        let white_positions = vec![(4, 1), (3, 2), (3, 4), (4, 4), (4, 6)];

        let mut mid_game_black = 0;
        let mut mid_game_white = 0;

        for pos in black_positions {
            mid_game_black |= 1 << (pos.1 * 8) + pos.0;
        }

        for pos in white_positions {
            mid_game_white |= 1 << (pos.1 * 8) + pos.0;
        }

        let mut game = Game::new();

        game.current_board.black = mid_game_black;
        game.current_board.white = mid_game_white;

        let legal_moves = game.legal_moves();
        assert_eq!(legal_moves.len(), 9);
        assert!(legal_moves.contains(&Move::from_col_row(2, 1).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(2, 2).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(3, 7).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(4, 0).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(4, 7).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(5, 0).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(5, 1).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(5, 6).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(5, 7).unwrap()));

        let mv = Move::from_col_row(2, 2).unwrap();
        game.play_next_turn(mv).unwrap();
        // ...x....
        // .xxBW...
        // .xBBBxx.
        // .xBBBB..
        // .xBWWBx.
        // ..BBBB..
        // .xxBWxx.
        // ...x....
        let new_black_positions = vec![
            (3, 1),
            (4, 2),
            (2, 3),
            (2, 2),
            (3, 2),
            (3, 3),
            (4, 3),
            (5, 3),
            (2, 4),
            (5, 4),
            (2, 5),
            (3, 5),
            (4, 5),
            (5, 5),
            (3, 6),
        ];
        let new_white_positions = vec![(4, 1), (3, 4), (4, 4), (4, 6)];

        let mut new_black = 0;
        let mut new_white = 0;
        for pos in new_black_positions {
            new_black |= 1 << (pos.1 * 8) + pos.0;
        }

        for pos in new_white_positions {
            new_white |= 1 << (pos.1 * 8) + pos.0;
        }

        assert_eq!(game.current_player, White);
        assert_eq!(game.current_board.black, new_black);
        assert_eq!(game.current_board.white, new_white);

        let legal_moves = game.legal_moves();
        assert_eq!(legal_moves.len(), 14);
        assert!(legal_moves.contains(&Move::from_col_row(1, 1).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(1, 2).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(1, 3).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(1, 4).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(1, 6).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(2, 1).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(2, 6).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(3, 0).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(3, 7).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(5, 2).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(5, 6).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(6, 2).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(6, 4).unwrap()));
        assert!(legal_moves.contains(&Move::from_col_row(6, 6).unwrap()));
    }

    #[test]
    fn test_legal_moves_known_late_game_state_with_no_moves() {
        let mut game = Game::new();
        let black_pieces: u64 = 44749019426896392;
        let white_pieces: u64 = 18374973456518217728;

        game.current_board.black = black_pieces;
        game.current_board.white = white_pieces;

        assert_eq!(game.to_move(), Black);
        let legal_moves = game.legal_moves();
        assert!(legal_moves.is_empty());
    }

    #[test]
    fn test_legal_moves_known_late_game_state_3_moves() {
        let mut game = Game::new();
        let black_pieces: u64 = 6863485832112635904;
        let white_pieces: u64 = 2323716670234820607;

        game.current_board.black = black_pieces;
        game.current_board.white = white_pieces;
        game.current_player = White;

        assert_eq!(game.to_move(), White);
        let legal_moves = game.legal_moves();
        assert_eq!(legal_moves.len(), 3);
    }

    #[test]
    fn test_apply_move() {
        let mut game = Game::new();
        let mv = Move::from_col_row(3, 2).unwrap();
        game.play_next_turn(mv).unwrap();
        let board = game.get_board();
        assert_eq!(board.black, 1 << 19 | 1 << 27 | 1 << 35 | 1 << 28);
        assert_eq!(board.white, 1 << 36);
    }

    #[test]
    fn test_score_initial_game_setup() {
        let mut game = Game::new();
        assert_eq!(game.score(), 0);
        let mv = Move::from_col_row(3, 2).unwrap();
        game.play_next_turn(mv).unwrap();
        assert_eq!(game.score(), 3);
    }

    #[test]
    fn test_move_new() {
        let position = 1 << 27;
        let mv = Move::new(position).unwrap();
        assert_eq!(mv, Move::from_col_row(3, 3).unwrap());
    }

    #[test]
    fn test_move_get_row_col() {
        let position = 1 << 27;
        let mv = Move::new(position).unwrap();
        let row = 3;
        let col = 3;
        assert_eq!(mv.get_col().unwrap(), col);
        assert_eq!(mv.get_row().unwrap(), row);
    }

    #[test]
    fn test_move_get_position() {
        let mv = Move::from_col_row(3, 3).unwrap();
        assert_eq!(mv.get_position().unwrap(), 1 << 27);
    }

    #[test]
    fn test_is_terminal() {
        let mut game = Game::new();
        // set bottom-right corner for white
        let white: u64 = 0 | (1 << 63);
        // set the whole board, unset the top-left and bottom-right corners
        let black: u64 = (!0 ^ 1) ^ (1 << 63);
        game.current_board.white = white;
        game.current_board.black = black;
        game.current_player = White;

        assert!(game.is_terminal());
    }

    #[test]
    fn test_legal_moves_known_board_state_that_failed() {
        // Failed board state:
        // To Move: White
        // Referee:      a b c d e f g h
        // Referee:    ********************
        // Referee:  1 * W W W W W W W B *
        // Referee:  2 * W B B B B W W B *
        // Referee:  3 * W W W W B W B B *
        // Referee:  4 * W W B B B B W B *
        // Referee:  5 * W W W W B B B B *
        // Referee:  6 * W W W W W W W B *
        // Referee:  7 * - - - - - W W - *
        // Referee:  8 * - - - - - - - - *
        // x: Legal moves
        let black: u64 = 0x80F0BCD09E80;
        let white: u64 = 0x60007F0F432F617F;

        let mut game = Game::new();
        game.current_board.black = black;
        game.current_board.white = white;
        game.current_player = White;

        assert_eq!(game.to_move(), White);
        let legal_moves = game.legal_moves();

        assert_eq!(legal_moves.len(), 0);
        for mv in &legal_moves {
            assert!(legal_moves.contains(&mv));
        }
    }

    #[test]
    fn test_game_hash() {
        let game1 = Game::new();
        let game2 = Game::new();

        let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();

        game1.hash(&mut hasher1);
        game2.hash(&mut hasher2);

        let game1_hash_value = hasher1.finish();
        let game2_hash_value = hasher2.finish();
        assert_eq!(game1_hash_value, game2_hash_value);
    }

    #[test]
    fn test_game_hash_mid_game() {
        let mut game1 = Game::new();
        let mut game2 = Game::new();

        for _ in 0..30 {
            let mv = game1.random_move();
            game1.play_next_turn(mv).unwrap();
            game2.play_next_turn(mv).unwrap();
        }

        let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();

        game1.hash(&mut hasher1);
        game2.hash(&mut hasher2);

        let game1_hash_value = hasher1.finish();
        let game2_hash_value = hasher2.finish();
        assert_eq!(game1_hash_value, game2_hash_value);
    }
}
