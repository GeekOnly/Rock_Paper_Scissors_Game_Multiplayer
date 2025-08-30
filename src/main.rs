mod application;
mod config;
mod domain;
mod infrastructure;

#[cfg(test)]
mod tests;

use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

use application::GameManager;
use config::ServerConfig;
use infrastructure::{rest_api, WebSocketHandler};



#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = ServerConfig::default();
    
    // Initialize game manager
    let game_manager = Arc::new(GameManager::new(config.game.into()));
    
    // Create WebSocket handler
    let ws_handler = WebSocketHandler::new(game_manager.clone());
    
    // WebSocket server
    let ws_config = config.websocket.clone();
    let ws_server = async move {
        let addr = format!("{}:{}", ws_config.host, ws_config.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("🎮 RPS Game Server running on {}", addr);
        info!("WebSocket: ws://{}", addr);

        while let Ok((stream, addr)) = listener.accept().await {
            info!("New connection from: {}", addr);
            let handler = ws_handler.clone();
            tokio::spawn(async move {
                if let Err(e) = handler.handle_connection(stream).await {
                    error!("WebSocket connection error: {}", e);
                }
            });
        }

        Ok::<(), anyhow::Error>(())
    };

    // REST API server
    let rest_config = config.rest_api.clone();
    let routes = rest_api::create_routes(game_manager);
    let rest_server = warp::serve(routes)
        .run(([127, 0, 0, 1], rest_config.port));

    info!("Health Check: http://{}:{}/health", rest_config.host, rest_config.port);
    info!("Stats: http://{}:{}/stats", rest_config.host, rest_config.port);

    // Run both servers concurrently
    tokio::try_join!(
        async { ws_server.await.map_err(|e| anyhow::anyhow!("WebSocket server error: {}", e)) },
        async { rest_server.await; Ok(()) }
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{GameChoice, GameConfig};
    use crate::application::GameRoom;

    #[test]
    fn test_game_choice_beats() {
        assert!(GameChoice::Rock.beats(&GameChoice::Scissors));
        assert!(GameChoice::Paper.beats(&GameChoice::Rock));
        assert!(GameChoice::Scissors.beats(&GameChoice::Paper));
        
        assert!(!GameChoice::Rock.beats(&GameChoice::Paper));
        assert!(!GameChoice::Paper.beats(&GameChoice::Scissors));
        assert!(!GameChoice::Scissors.beats(&GameChoice::Rock));
        
        assert!(!GameChoice::Rock.beats(&GameChoice::Rock));
    }

    #[test]
    fn test_game_room_creation() {
        let config = GameConfig::default();
        let room = GameRoom::new("test-room".to_string(), config);
        assert_eq!(room.id, "test-room");
        assert_eq!(room.status, crate::domain::GameStatus::Waiting);
        assert_eq!(room.current_round, 1);
        assert_eq!(room.config.max_rounds, 3);
    }
}