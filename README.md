# HERB - Hayden's Excellent Reversi Bot

An AI that plays Othello/Reversi

HERB is an Othello/Reversi bot written in Rust. Herb uses a modified Monte Carlo
Tree Search (MCTS) algorithm that runs in parallel on multiple threads. The bulk of the
logic is in the MCTS module and the Othello module.

# Setup

Install the [Rust](https://www.rust-lang.org/) compiler and associated tools:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Build the project using cargo (installed with rust)

```bash
cargo build
```

Run `herb` to interface with the referee, run `herbvrandom` to see Herb play against an
opponent making random moves:

```bash
cargo run --bin herb
cargo run --bin herbvrandom
```

Cleanup the build directory:

```bash
cargo clean
```

# Code Structure

## Rust

- main.rs - runs a loop that plays through a game with the referee
- lib.rs - defines the interfaces and structs used to interface with the referee and Herb
- config.rs - configuration settings for Herb and the Monte Carlo Search
- mcts.rs - Monte Carlo Tree Search implementation
- othello.rs - Othello game engine

Note - I also have a minimax.rs and minimaxab.rs that implement
Minimax and Minimax with Alpha-Beta Pruning. Herb can be setup to use them pretty easily, but it currently requires
changing the code.

# Other

In lib.rs there is a `Player` trait and a `GameInterface` trait that need to be
implemented so that Herb can talk to different players/protocols. The only
implmentation so far is the the drmecref.rs file so that Herb can play against
opponents interfacing with Dr. Cameron's Referee program.
