use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use super::messages::ServerMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: String,
}

pub struct Player {
    pub id: String,
    pub sender: mpsc::UnboundedSender<ServerMessage>,
}

impl Player {
    pub fn new(id: String, sender: mpsc::UnboundedSender<ServerMessage>) -> Self {
        Self { id, sender }
    }

    pub async fn send_message(&self, message: &ServerMessage) -> Result<()> {
        self.sender
            .send(message.clone())
            .map_err(|_| anyhow::anyhow!("Failed to send message to player {}", self.id))?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub total_games: u32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            wins: 0,
            losses: 0,
            draws: 0,
            total_games: 0,
        }
    }
}