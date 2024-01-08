//! A Monte Carlo Tree Search Implementation
//!
//!
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use crate::config::MctsConfig;
use serde::{Deserialize, Serialize};

use crate::drmecref::DrMecRef;
use crate::othello::Move::Pass;
use crate::othello::{Color, Game, Move};

/// Represents a Monte Carlo Search Tree.
///
/// The tree is represented as a map of game states to tree nodes.
pub struct Tree {
    pub(crate) config: MctsConfig,
    pub(crate) map: HashMap<u64, Node>,
    pub(crate) search_iterations: u64,
}

impl Tree {
    /// Create a new MCTS Tree using a default [`MctsConfig`].
    pub fn new() -> Self {
        let config = MctsConfig::default();
        Tree::from_config(config)
    }

    /// Create a new MCTS Tree using the given [`MctsConfig`].
    pub fn from_config(config: MctsConfig) -> Self {
        Tree {
            config,
            map: HashMap::new(),
            search_iterations: 0,
        }
    }

    /// Merge the given tree with this tree.
    ///
    /// A merge adds the values from any [`Node`]s the trees
    /// have in common, and inserts any [`Node`]s from the other tree
    /// that are not in this tree
    pub fn merge(&mut self, other: Tree) {
        for (key, value) in other.map {
            self.map
                .entry(key)
                .and_modify(|node| {
                    node.visits += value.visits;
                    node.wins += value.wins;
                })
                .or_insert(value);
        }
        self.search_iterations += other.search_iterations;
    }
}

impl Tree {
    /// Perform one full iteration of a Monte Carlo Tree Search. Each
    /// call to search grows the tree by one node. It uses the [`Game`] instances
    /// as the nodes in the tree with the legal [`Move`]s for the given game
    /// state as the edges. The 'tree' is traversed by calling 'play_next_turn(some_move)' on
    /// a [`Game`] node. This means that there are no 'parent' pointers so the search
    /// uses a stack to keep track of which nodes are visited during the search process.
    ///
    /// It follows the classic MCTS algorithm:
    /// 1. Select a leaf node by traversing the tree until a leaf node is found.
    ///     - The selection process uses the UCB1 formula to decide which branch of the tree to go down at each step.
    ///     - Nodes are pushed onto a stack as they are selected so we can follow the path back up in the backpropagation step.
    /// 2. Expand the tree by adding a child of the selected leaf to the tree.
    ///     - The tree is expanded by stepping down a level from the selected leaf and adding a new node to the tree.
    /// 3. Simulate to the end of the game starting the newly created node, keeping track of the winner
    /// 4. Backpropagate up the tree updating the `wins` and `visits` values at each node.
    ///
    /// A game does not have to progress strictly in sequence for the tree to work, you can pass in a [`Game`]
    /// in any state and the tree will grow starting from that 'node'. The `wins` and `visits` are
    /// stored in a [`Node`] struct, a [`HashMap`] is used to map a [`Game`] to a [`Node`].
    pub fn search(&mut self, game: Game) {
        if !game.is_over() {
            let (leaf, mut stack) = self.select(game);
            let child = self.expand(leaf);
            if child != game {
                stack.push(child);
            }
            let winner = self.simulate(child);
            self.backpropagate(game.to_move(), winner, stack);
            self.search_iterations += 1;
        }
    }

    /// Select a leaf node by walking the tree, pushing game states onto the stack
    /// as we pass them.
    ///
    /// Returns a two-tuple with the first element being the selected leaf node and the second
    /// is the stack of nodes that were visited on the way to the selected node.
    fn select(&self, game: Game) -> (Game, Vec<Game>) {
        let mut stack = Vec::new();
        let mut sim_game = game;
        while !sim_game.is_over() && !self.leaf_p(sim_game) {
            stack.push(sim_game);
            let mv = self.ucb1(sim_game);
            sim_game.play_next_turn(mv).unwrap();
        }
        (sim_game, stack)
    }

    /// Expands the tree by creating a new child node from the passed in leaf node.
    /// Returns the new child node.
    fn expand(&self, leaf: Game) -> Game {
        let legal_moves = leaf.legal_moves();
        for mv in legal_moves {
            let mut sim_game = leaf;
            sim_game.play_next_turn(mv).unwrap();
            if !self.map.contains_key(&sim_game.get_hash()) {
                return sim_game;
            }
        }
        leaf
    }

    /// Simulates to the end of the given game and reports the winner.
    /// If the winner is `None` the game ended in a draw, otherwise
    /// the returned `Some(Color)` will contain the winner.
    fn simulate(&self, mut game: Game) -> Option<Color> {
        while !game.is_over() {
            let mut mv = self.best_move(game, false);
            if mv == Pass {
                mv = game.random_move()
            }
            game.play_next_turn(mv).unwrap();
        }
        game.winner()
    }

    /// Walk back up the tree by popping nodes off the stack. 'Visit' each node updating the
    /// `wins` and `visits` if the [`Node`] is in the tree or inserting a new node.
    fn backpropagate(&mut self, player: Color, winner: Option<Color>, stack: Vec<Game>) {
        let result_value = match winner {
            // A draw is worth half a win.
            None => 0.5,
            Some(winner) if winner == player => 1.0,
            _ => 0.0,
        };
        for game in stack {
            self.map
                .entry(game.get_hash())
                .and_modify(|node| {
                    node.wins += result_value;
                    node.visits += 1.0;
                })
                .or_insert_with(|| Node::first_visit(result_value == 1.0));
        }
    }

    /// The UCB1 formula for deciding which child nodes to visit during the select phase
    /// of MCTS.
    fn ucb1(&self, game: Game) -> Move {
        let mut best_move = game.random_move();
        let mut best_value = f64::MIN;

        let parent_visits = self
            .map
            .get(&game.get_hash())
            .map_or(1.0, |node| node.visits);

        let legal_moves = game.legal_moves();

        for mv in legal_moves {
            let mut sim_game = game;
            sim_game.play_next_turn(mv).unwrap();

            let node = self
                .map
                .get(&sim_game.get_hash())
                .map_or_else(Node::cold_start, |n| *n);

            let visits = node.visits.max(1.0);

            let exploitation = if node.visits > 0.0 {
                node.wins / visits
            } else {
                0.0
            };

            let exploration =
                self.config.exploration_factor * ((parent_visits.ln() + 1e-5) / visits).sqrt();

            let ucb1_value = exploitation + exploration;

            if ucb1_value > best_value {
                best_value = ucb1_value;
                best_move = mv;
            }
        }
        best_move
    }

    /// Picks the best move according to various attributes of the nodes that are
    /// in the tree.
    pub fn best_move(&self, game: Game, last: bool) -> Move {
        let mut best_move = Pass;
        let mut best_value = f64::MIN;
        let legal_moves = game.legal_moves();

        for mv in legal_moves {
            let mut sim_game = game;
            sim_game.play_next_turn(mv).unwrap();

            let value = self.evaluate(sim_game);

            if last {
                DrMecRef::comment(format!("MCTS: Considering Move {}, Value: {}", mv, value));
            }

            if value > best_value {
                best_value = value;
                best_move = mv;
            }
        }

        best_move
    }

    /// This is the evaluation function that ultimately determines what
    /// the best move is based on the nodes stored in the tree.
    /// The game state is evaluated from the perspective that the game
    /// passed is the result of a legal move from some player. The score that
    /// is returned will be high if it is a desirable state to move to from the
    /// calling player's perspective.
    fn evaluate(&self, game: Game) -> f64 {
        let node = match self.map.get(&game.get_hash()) {
            None => Node::cold_start(),
            Some(node) => *node,
        };

        let (black_corners, white_corners) = game.num_corners_held();
        let (own_corners_held, opponent_corners_held) = match game.to_move() {
            Color::Black => (white_corners as f64, black_corners as f64),
            Color::White => (black_corners as f64, white_corners as f64),
        };

        let (black_edges, white_edges) = game.num_edges_held();
        let (own_edges_held, opponent_edges_held) = match game.to_move() {
            Color::Black => (white_edges as f64, black_edges as f64),
            Color::White => (black_edges as f64, white_edges as f64),
        };

        let (black_x_moves, white_x_moves) = game.num_x_moves_held();
        let (own_x_moves_held, opponent_x_moves_held) = match game.to_move() {
            Color::Black => (white_x_moves as f64, black_x_moves as f64),
            Color::White => (black_x_moves as f64, white_x_moves as f64),
        };

        let (black_diagonals, white_diagonals) = game.diagonals_held();
        let (own_diagonals_held, opponent_diagonals_held) = match game.to_move() {
            Color::Black => (white_diagonals as f64, black_diagonals as f64),
            Color::White => (black_diagonals as f64, white_diagonals as f64),
        };

        let (black_center_4, white_center_4) = game.center_4_held();
        let (own_center_4_held, opponent_center_4_held) = match game.to_move() {
            Color::Black => (white_center_4 as f64, black_center_4 as f64),
            Color::White => (black_center_4 as f64, white_center_4 as f64),
        };

        let (black_inner_board, white_inner_board) = game.inner_board_held();
        let (own_inner_board_held, opponent_inner_board_held) = match game.to_move() {
            Color::Black => (white_inner_board as f64, black_inner_board as f64),
            Color::White => (black_inner_board as f64, white_inner_board as f64),
        };

        let corners_difference = own_corners_held - opponent_corners_held;
        let edges_difference = own_edges_held - opponent_edges_held;
        let center_4_difference = own_center_4_held - opponent_center_4_held;
        let inner_board_difference = own_inner_board_held - opponent_inner_board_held;
        let x_moves_difference = opponent_x_moves_held - own_x_moves_held;
        let diagonals_difference = own_diagonals_held - opponent_diagonals_held;

        let opponent_mobility = game.mobility() as f64;

        let normalized_visits = 10.0 * 1.0 / (1.0 + (node.visits * -1.0).exp());
        let win_ratio = node.ratio();

        let mut value: f64 = normalized_visits;
        value += 10.0 * win_ratio;
        value += 2.0 * corners_difference;
        value += 1.5 * edges_difference;
        value += 1.75 * diagonals_difference;
        value += center_4_difference;
        value += inner_board_difference;
        value -= 1.5 * opponent_mobility;
        value -= x_moves_difference;
        value
    }

    /// Determines if the given game is a 'Leaf' node in the MCTS Tree.
    /// A leaf is any node that has any unexplored children.
    fn leaf_p(&self, game: Game) -> bool {
        if game.is_over() {
            return true;
        }
        let legal_move = game.legal_moves();
        for mv in legal_move {
            let mut sim_game = game;
            sim_game.play_next_turn(mv).unwrap();
            if self.map.contains_key(&sim_game.get_hash()) {
                return false;
            }
        }
        true
    }
}

impl Debug for Tree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Im a tree!")
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

/// Holds the visits and wins for a node in the tree
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub(crate) visits: f64,
    pub(crate) wins: f64,
}

impl Node {
    /// Create a new node
    pub fn new() -> Self {
        Node {
            visits: 0.0,
            wins: 0.0,
        }
    }

    /// Create a new node with visits set to `1.0` and wins set to `0.0` or `1.0`
    /// depending on the passed in win bool.
    pub fn first_visit(win: bool) -> Self {
        Node {
            visits: 1.0,
            wins: if win { 1.0 } else { 0.0 },
        }
    }

    /// Creates a new node with 1 visit and 0.5 wins. Used in the UCB1 formula to avoid dividing by
    /// zero if a node hasn't been visited yet.
    pub fn cold_start() -> Self {
        Node {
            visits: 1.0, // Start with a visit to avoid division by zero in UCB1
            wins: 0.5,   // Start with a draw to give a fair initial win rate
        }
    }

    /// Returns the ratio of wins to visits
    pub fn ratio(&self) -> f64 {
        self.wins / self.visits
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_search_iteration() {
        let mut tree = Tree::new();
        let game = Game::new();
        tree.search(game);
        let mv = tree.best_move(game, false);

        let _legal_moves = game.legal_moves();
        let mut sim_game = game;
        sim_game.play_next_turn(mv).unwrap();
        assert!(tree.map.contains_key(&sim_game.get_hash()));

        if let Some(node) = tree.map.get(&sim_game.get_hash()) {
            assert_ne!(node.visits, 0.0);
        } else {
            panic!("Node not found in tree!");
        }
    }

    #[test]
    fn test_merge() {
        let mut t1 = Tree::new();
        let mut t2 = Tree::new();
        let game = Game::new();

        t1.search(game);

        for _ in 0..10 {
            t2.search(game);
        }

        let t1_map_pre_merge = &t1.map.clone();
        let t2_map_pre_merge = &t2.map.clone();

        t1.merge(t2);

        for (game, t2_node) in t2_map_pre_merge {
            assert!(t1.map.contains_key(game));
            if let Some(t1_node) = t1.map.get(game) {
                if let Some(t1_pre_merge_node) = t1_map_pre_merge.get(game) {
                    assert_eq!(t1_node.visits, t1_pre_merge_node.visits + t2_node.visits);
                    assert_eq!(t1_node.wins, t1_pre_merge_node.wins + t2_node.wins);
                }
            }
        }
    }
}
