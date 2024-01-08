//! Configuration structs for Herb and the mcts trees.
//!
//! Herb expects a configuration file. It does not matter what it is named,
//! but it must be in json format.
//! ```json
//! {
//!     "max_time": 100.0,
//!     "log": true,
//!     "mcts_config": {
//!         "exploration_factor": 1.418
//!     }
//! }
//! ```
//! None of the options are required, if any are omitted they will be filled with sensible defaults.
//!
//! # List of Configuration Settings
//! - max_time: float total time limit for a game in seconds
//! - log: boolean output logging info
//! - mcts_config: Configuration setting for the [`mcts`] module.
//!     - exploration_factor: float used in UCB1 to determine when to explore unknown parts of the tree.
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::drmecref::DrMecRef;

/// Configuration Settings for [`Herb`]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default)]
    pub max_time: f64,
    #[serde(default)]
    pub log: bool,
    #[serde(default)]
    pub mcts_config: MctsConfig,
}

/// Configuration settings for the MCTS [`Tree`]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MctsConfig {
    #[serde(default)]
    pub exploration_factor: f64,
}

impl Config {
    /// Create a new Config from the given json config file.
    /// If the reading or parsing the given config file fails,
    /// a default Config is used.
    pub fn new(config_file: &str) -> Self {
        if let Ok(config) = File::open(config_file) {
            Config::parse(config)
        } else {
            Config::default()
        }
    }

    /// Create a new Config by parsing the given json config file
    fn parse(config_file: File) -> Self {
        let mut config_str = String::new();
        let mut config_file = config_file;

        if config_file.read_to_string(&mut config_str).is_err() {
            DrMecRef::comment("Failed to read the configuration file; using defaults.");
            return Config::default();
        }

        match serde_json::from_str::<Config>(&config_str) {
            Ok(parsed_config) => parsed_config,
            Err(e) => {
                DrMecRef::comment(format!(
                    "Failed to parse the configuration file: {}; using defaults.",
                    e
                ));
                Config::default()
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_time: 120.0,
            mcts_config: MctsConfig::default(),
            log: true,
        }
    }
}

impl Default for MctsConfig {
    fn default() -> Self {
        MctsConfig {
            exploration_factor: std::f64::consts::SQRT_2,
        }
    }
}
