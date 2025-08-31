use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub websocket: WebSocketConfig,
    pub rest_api: RestApiConfig,
    pub game: GameConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub connection_timeout_ms: u64,
    pub message_timeout_ms: u64,
    pub keepalive_interval_ms: u64,
    pub max_frame_size: usize,
    pub max_message_size: usize,
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
    pub move_timeout_ms: u64,
    pub cleanup_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub worker_threads: Option<usize>,
    pub max_blocking_threads: usize,
    pub thread_stack_size: usize,
    pub channel_buffer_size: usize,
    pub gc_interval_ms: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            websocket: WebSocketConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                max_connections: 25000, // Increased for extreme testing
                connection_timeout_ms: 2000, // Faster timeout for high load
                message_timeout_ms: 500,     // Ultra-fast message timeout
                keepalive_interval_ms: 10000, // More frequent keepalive
                max_frame_size: 16 * 1024,   // Smaller frames for efficiency
                max_message_size: 256 * 1024, // Smaller messages
            },
            rest_api: RestApiConfig {
                host: "0.0.0.0".to_string(),
                port: 8081,
            },
            game: GameConfig {
                max_rounds: 3,
                min_players: 2,
                max_players: 2,
                move_timeout_ms: 15000,
                cleanup_interval_ms: 30000,
            },
            performance: PerformanceConfig {
                worker_threads: Some(16), // More worker threads
                max_blocking_threads: 2048, // More blocking threads
                thread_stack_size: 1024 * 1024, // Smaller stack for more threads
                channel_buffer_size: 4096, // Larger buffers
                gc_interval_ms: 10000, // More frequent GC
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