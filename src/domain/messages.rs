use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{GameChoice, PlayerInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ClientMessage {
    Connect {
        #[serde(rename = "playerId")]
        player_id: Option<String>,
    },
    FindMatch,
    PlayerMove { choice: GameChoice },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ServerMessage {
    Connected {
        #[serde(rename = "playerId")]
        player_id: String,
    },
    Matchmaking {
        matched: bool,
        waiting: Option<bool>,
        #[serde(rename = "roomId")]
        room_id: Option<String>,
    },
    GameStart {
        #[serde(rename = "roomId")]
        room_id: String,
        players: Vec<PlayerInfo>,
        #[serde(rename = "maxRounds")]
        max_rounds: u32,
    },
    RoundResult {
        round: u32,
        winner: Option<String>,
        moves: HashMap<String, GameChoice>,
        scores: HashMap<String, u32>,
    },
    NextRound { round: u32 },
    GameEnd {
        winner: Option<String>,
        #[serde(rename = "finalScores")]
        final_scores: HashMap<String, u32>,
    },
    PlayerLeft {
        #[serde(rename = "playerId")]
        player_id: String,
    },
    Error { message: String },
}