//! Minimax with Alpha-Beta Pruning
use crate::drmecref::DrMecRef;
use crate::othello::Color::{Black, White};
use crate::othello::Move::Pass;
use crate::othello::{Color, Game, Move, CORNERS, EDGES};
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// maximum depth for the tree traversal
const MAX_DEPTH: i32 = 4;
// turn when the early game starts
const EARLY_GAME: usize = 10;
// turn when the mid game starts
const MID_GAME: usize = 35;
// turn when the end game starts
const END_GAME: usize = 60;
// time limit for search
const TIME_LIMIT_MILLIS: u64 = 100;
const TIME_MULTIPLIER: u64 = 10;

const CORNER_MULTIPLIER: i32 = 5;
const EDGE_MULTIPLIER: i32 = 2;

enum PlayerType {
    Max,
    Min,
}

pub fn minimax(game_state: Game, max_player: Color) -> Move {
    let mut best_move = Pass;
    let start_time = Instant::now();
    let time_limit = TIME_LIMIT_MILLIS;

    for depth in 1..MAX_DEPTH {
        // if game_state.get_turn() >= EARLY_GAME && !game_state.get_turn() >= END_GAME {
        //     time_limit *= TIME_MULTIPLIER;
        // }
        if Instant::now() >= start_time + Duration::from_millis(time_limit) {
            // DrMecRef::comment(format!("MINIMAX: Stopping search at depth {}", depth));
            break;
        }
        let (_value, action, _ply) = value(
            game_state,
            0,
            i32::MIN,
            i32::MAX,
            max_player,
            PlayerType::Max,
            depth + 1,
            Instant::now(),
        );
        best_move = action;
    }

    best_move
}

fn value(
    game_state: Game,
    ply: i32,
    mut alpha: i32,
    mut beta: i32,
    max_player: Color,
    player_type: PlayerType,
    max_depth: i32,
    start_time: Instant,
) -> (i32, Move, i32) {
    if game_state.is_terminal() || ply >= max_depth {
        let legal_moves = sort_moves(game_state);
        let last_move = *legal_moves.front().unwrap_or(&Pass);
        let score = evaluate_state(game_state, max_player);
        return (score, last_move, ply);
    }

    let mut v = match player_type {
        PlayerType::Max => i32::MIN,
        PlayerType::Min => i32::MAX,
    };

    let actions = sort_moves(game_state);
    let mut mv = *actions.front().unwrap_or(&Pass);

    for action in actions {
        let mut sim_game = game_state;
        sim_game.play_next_turn(action).unwrap();

        let mut value2 = 0;
        let _action2 = Pass;
        let mut ply = ply;
        if Instant::now() < start_time + Duration::from_millis(TIME_LIMIT_MILLIS + 1000) {
            let (value2, _action2, ply) = match player_type {
                PlayerType::Max => value(
                    sim_game,
                    ply + 1,
                    alpha,
                    beta,
                    max_player,
                    PlayerType::Min,
                    max_depth,
                    start_time,
                ),
                PlayerType::Min => value(
                    sim_game,
                    ply + 1,
                    alpha,
                    beta,
                    max_player,
                    PlayerType::Max,
                    max_depth,
                    start_time,
                ),
            };
        } else {
            let (value2, _action2) = match player_type {
                PlayerType::Max => match max_player {
                    White => (evaluate_state(game_state, Black), action),
                    Black => (evaluate_state(game_state, White), action),
                },
                PlayerType::Min => (evaluate_state(game_state, max_player), action),
            };
        };

        match player_type {
            PlayerType::Max => {
                if value2 > v {
                    v = value2;
                    mv = action;
                    alpha = i32::max(alpha, v);
                }
                if v >= beta {
                    return (v, mv, ply);
                }
            }
            PlayerType::Min => {
                if value2 < v {
                    v = value2;
                    mv = action;
                    beta = i32::min(beta, v);
                }
                if v <= alpha {
                    return (v, mv, ply);
                }
            }
        }
    }

    (v, mv, ply)
}

fn evaluate_state(game_state: Game, max_player: Color) -> i32 {
    let mut score: i32 = 0;

    let current_moves = game_state.legal_moves();

    if game_state.get_turn() < MID_GAME {
        if game_state.to_move() == max_player {
            score = current_moves.len() as i32;
        } else {
            for mv in &current_moves {
                let mut sim_game = game_state;
                sim_game.play_next_turn(*mv).unwrap();
                let sim_moves = sim_game.legal_moves();
                let mut move_score = sim_moves.len() as i32;

                if CORNERS.contains(mv) {
                    move_score *= CORNER_MULTIPLIER;
                }
                if EDGES.contains(mv) {
                    move_score *= EDGE_MULTIPLIER;
                }

                if score > move_score {
                    score = move_score;
                }
            }
        }
    } else {
        score = game_state.score();
        if max_player == White {
            score *= -1;
        }
    }
    score
}

fn sort_moves(game: Game) -> VecDeque<Move> {
    let mut game_deque: VecDeque<Game> = VecDeque::new();
    let mut move_deque: VecDeque<Move> = VecDeque::new();

    for mv in game.legal_moves() {
        let mut sim_game = game;
        sim_game.play_next_turn(mv).unwrap();

        if game_deque.is_empty() {
            game_deque.push_back(sim_game);
            move_deque.push_back(mv);
            continue;
        }

        let mut inserted = false;
        for (index, existing_game) in game_deque.iter().enumerate() {
            if sim_game > *existing_game {
                game_deque.insert(index, sim_game);
                move_deque.insert(index, mv);
                inserted = true;
                break;
            }
        }

        if !inserted {
            game_deque.push_back(sim_game);
            move_deque.push_back(mv);
        }
    }

    move_deque
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        evaluate_state(*self, self.to_move()) == evaluate_state(*other, other.to_move())
    }
}

impl PartialOrd for Game {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Game {
    fn cmp(&self, other: &Self) -> Ordering {
        let diff = evaluate_state(*self, self.to_move()) - evaluate_state(*other, other.to_move());
        if diff > 0 {
            Ordering::Greater
        } else if diff == 0 {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    }
}
