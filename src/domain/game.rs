use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GameChoice {
    Rock,
    Paper,
    Scissors,
}

impl GameChoice {
    pub fn beats(&self, other: &GameChoice) -> bool {
        matches!(
            (self, other),
            (GameChoice::Rock, GameChoice::Scissors)
                | (GameChoice::Paper, GameChoice::Rock)
                | (GameChoice::Scissors, GameChoice::Paper)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GameStatus {
    Waiting,
    Playing,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMove {
    pub choice: GameChoice,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GameResult {
    pub round: u32,
    pub winner: Option<String>,
    pub moves: HashMap<String, GameChoice>,
    pub scores: HashMap<String, u32>,
}

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub max_rounds: u32,
    pub min_players: usize,
    pub max_players: usize,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            max_rounds: 3,
            min_players: 2,
            max_players: 2,
        }
    }
}