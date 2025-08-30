use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub websocket: WebSocketConfig,
    pub rest_api: RestApiConfig,
    pub game: GameConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestApiConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub max_rounds: u32,
    pub min_players: usize,
    pub max_players: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            websocket: WebSocketConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            rest_api: RestApiConfig {
                host: "127.0.0.1".to_string(),
                port: 8081,
            },
            game: GameConfig {
                max_rounds: 3,
                min_players: 2,
                max_players: 2,
            },
        }
    }
}

impl From<GameConfig> for crate::domain::GameConfig {
    fn from(config: GameConfig) -> Self {
        Self {
            max_rounds: config.max_rounds,
            min_players: config.min_players,
            max_players: config.max_players,
        }
    }
}