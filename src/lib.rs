//! HERB - Hayden's Excellent Reversi Bot
//!
//! An AI that plays Othello/Reversi
//!
//! Created by: Hayden Holbrook
//!
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use rayon::current_num_threads;
use rayon::prelude::*;

use crate::config::Config;
use crate::drmecref::DrMecRef;
use crate::mcts::Tree;
use crate::othello::Move::Pass;
use crate::othello::{Color, Game, Move};

pub mod config;
pub mod drmecref;
pub mod mcts;
pub mod othello;

// Time allocations per turn as a percentage of the remaining time
const TIME_ALLOCATIONS: [f64; 70] = [
    0.015, 0.015, 0.015, 0.015, 0.025, 0.025, 0.025, 0.025, 0.025, 0.025, 0.048, 0.048, 0.048,
    0.048, 0.048, 0.048, 0.050, 0.051, 0.052, 0.053, 0.044, 0.045, 0.049, 0.049, 0.049, 0.051,
    0.053, 0.055, 0.057, 0.059, 0.060, 0.060, 0.061, 0.062, 0.063, 0.064, 0.065, 0.065, 0.065,
    0.065, 0.167, 0.168, 0.169, 0.169, 0.171, 0.172, 0.173, 0.175, 0.180, 0.180, 0.181, 0.187,
    0.196, 0.199, 0.060, 0.060, 0.060, 0.060, 0.060, 0.060, 0.060, 0.060, 0.060, 0.060, 0.060,
    0.060, 0.060, 0.060, 0.060, 0.060,
];

pub struct Herb {
    config: Config,
    mcts: Tree,
    search_iterations: u64,
    time_remaining: f64,
}

impl Herb {
    /// Create a new instance of Herb using the given [`Config`].
    pub fn new(config: Config) -> Herb {
        let tree = Tree::from_config(config.mcts_config.clone());
        let max_time = config.max_time;
        if config.log {
            DrMecRef::comment(format!("{:?}", config));
        }
        Herb {
            config,
            mcts: tree,
            search_iterations: 0,
            time_remaining: max_time,
        }
    }

    /// Returns the total number of search iterations performed by this tree.
    pub fn search_iterations(&self) -> u64 {
        self.search_iterations
    }

    /// Calculate the time allocation for a turn based on the given game state.
    fn dynamic_time_limit(&mut self, game: Game) -> Duration {
        let turn_num = game.get_turn();
        let time_for_turn = self.time_remaining * TIME_ALLOCATIONS[turn_num];
        self.time_remaining -= time_for_turn;
        Duration::from_secs_f64(time_for_turn)
    }

    /// Get Herb's move for the given game. Herb assumes that `game.to_move()` is Herb's color
    /// and will choose a move from the legal moves available for the given game.
    fn get_move(&mut self, game: Game) -> Move {
        let start_time = Instant::now();
        let time_limit = start_time + self.dynamic_time_limit(game);
        // self.single_threaded_search(game, time_limit);
        let trees = self.multi_threaded_search(game, time_limit);

        trees.into_iter().for_each(|tree| {
            self.mcts.merge(tree);
        });

        self.mcts.best_move(game, true)
    }

    /// Perform the MCTS algorithm in a single thread until the time_limit is reached.
    /// The search will use the given game as the starting point in the tree.
    fn _single_threaded_search(&mut self, game: Game, time_limit: Instant) {
        while Instant::now() <= time_limit {
            self.mcts.search(game)
        }
    }

    /// Perform the MCTS algorithm in the maximum number of threads equal to the number of cpus
    /// available on whatever machine Herb is running on.
    ///
    /// Return forest, a `Vec<Tree>`, all rooted at the given game.
    fn multi_threaded_search(&mut self, game: Game, time_limit: Instant) -> Vec<Tree> {
        let num_trees = current_num_threads();

        let search_counters: Vec<_> = (0..num_trees).map(|_| AtomicUsize::new(0)).collect();

        // kick off the threads
        let trees: Vec<_> = (0..num_trees)
            .into_par_iter()
            .enumerate()
            .map(|(index, _)| {
                let mut local_tree = Tree::new();
                let local_game = game;
                let counter = &search_counters[index];

                while Instant::now() <= time_limit {
                    local_tree.search(local_game);
                    counter.fetch_add(1, Ordering::SeqCst);
                }
                local_tree
            })
            .collect();

        if self.config.log {
            let mut total = 0;
            for (index, counter) in search_counters.iter().enumerate() {
                DrMecRef::comment(format!(
                    "Herb: Thread {} completed {} iterations",
                    index,
                    counter.load(Ordering::SeqCst)
                ));
                total += counter.load(Ordering::SeqCst);
            }
            DrMecRef::comment(format!(
                "Herb: Total search iterations this turn: {}",
                total
            ));
            self.search_iterations += total as u64;
        }

        trees
    }
}

impl Player for Herb {
    /// Get Herb's next move for the given game.
    fn get_next_move(&mut self, game_state: Game) -> Move {
        let legal_moves = game_state.legal_moves();
        if !legal_moves.is_empty() {
            let mv = self.get_move(game_state);
            return if legal_moves.contains(&mv) {
                if self.config.log {
                    DrMecRef::comment(format!(
                        "Herb: Total Search iterations this game: {}",
                        self.search_iterations
                    ));
                    DrMecRef::comment(format!("Herb: Sending move: {}", mv));
                }
                mv
            } else {
                if self.config.log {
                    DrMecRef::comment("Herb: Got illegal move from search! Sending random move!!");
                }
                *legal_moves.first().unwrap_or(&Pass)
            };
        } else {
            Pass
        }
    }
}

pub trait GameInterface {
    /// Send the given [`Move`] and [`Color`].
    fn send_move(&self, mv: Move, color: Color) -> io::Result<()>;

    /// Receive a Move.
    fn receive_move(&self) -> io::Result<Move>;
}

pub trait Player {
    fn get_next_move(&mut self, game_state: Game) -> Move;
}
