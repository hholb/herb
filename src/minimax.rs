use crate::othello::Color::Black;
use crate::othello::Move::Pass;
use crate::othello::{Color, Game, Move};

const MAX_DEPTH: i32 = 2;

pub struct Minimax {
    max_player: Color,
}

impl Minimax {
    pub fn new(max_player: Color) -> Self {
        Minimax { max_player }
    }

    pub fn minimax(&self, game_state: Game) -> Move {
        let (_value, action) = self.max_value(game_state, 0);
        action
    }

    fn max_value(&self, game_state: Game, ply: i32) -> (i32, Move) {
        if game_state.is_terminal() || ply >= MAX_DEPTH {
            let legal_moves = game_state.legal_moves();
            let last_move = *legal_moves.first().unwrap_or(&Pass);
            let score = self.evaluate_state(game_state);
            return (score, last_move);
        }

        let mut v = i32::MIN;
        let actions = game_state.legal_moves();
        let mut mv = *actions.first().unwrap_or(&Pass);
        for action in actions {
            let mut sim_game = game_state;
            sim_game.play_next_turn(action).unwrap();
            let (value2, _action2) = self.min_value(sim_game, ply + 1);
            if value2 > v {
                v = value2;
                mv = action;
            }
        }

        (v, mv)
    }

    fn min_value(&self, game_state: Game, ply: i32) -> (i32, Move) {
        if game_state.is_over() || ply >= MAX_DEPTH {
            let legal_moves = game_state.legal_moves();
            let last_move = *legal_moves.first().unwrap_or(&Pass);
            let score = self.evaluate_state(game_state);
            return (score, last_move);
        }

        let mut v = i32::MAX;
        let actions = game_state.legal_moves();
        let mut mv = *actions.first().unwrap_or(&Pass);
        for action in actions {
            let mut sim_game = game_state;
            sim_game.play_next_turn(action).unwrap();
            let (value2, _action2) = self.max_value(sim_game, ply + 1);
            if value2 < v {
                v = value2;
                mv = action;
            }
        }

        (v, mv)
    }

    fn evaluate_state(&self, game_state: Game) -> i32 {
        let mut score: i32 = 0;
        if game_state.get_turn() < 30 {
            if game_state.to_move() == self.max_player {
                score = game_state.legal_moves().len() as i32;
            } else {
                for mv in game_state.legal_moves() {
                    let mut sim_game = game_state;
                    sim_game.play_next_turn(mv).unwrap();
                    let sim_moves = sim_game.legal_moves();
                    if score > sim_moves.len() as i32 {
                        score = sim_moves.len() as i32;
                    }
                }
            }
        } else {
            score = game_state.score();
            match self.max_player {
                Color::Black => {}
                Color::White => {
                    score *= -1;
                }
            }
        }
        score
    }
}

impl Default for Minimax {
    fn default() -> Self {
        Self::new(Black)
    }
}
