use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info};
use uuid::Uuid;

use crate::application::GameManager;
use crate::domain::{ClientMessage, Player, ServerMessage};

#[derive(Clone)]
pub struct WebSocketHandler {
    game_manager: Arc<GameManager>,
}

impl WebSocketHandler {
    pub fn new(game_manager: Arc<GameManager>) -> Self {
        Self { game_manager }
    }

    pub async fn handle_connection(&self, raw_stream: TcpStream) -> Result<()> {
        let ws_stream = accept_async(raw_stream).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        let mut player_id: Option<String> = None;

        // Create a channel for sending messages to this client
        let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

        info!("New WebSocket client connected");

        // Spawn a task to handle outgoing messages
        let sender_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                let json = match serde_json::to_string(&message) {
                    Ok(json) => json,
                    Err(e) => {
                        error!("Failed to serialize message: {}", e);
                        continue;
                    }
                };

                if let Err(e) = ws_sender.send(Message::Text(json)).await {
                    error!("Failed to send WebSocket message: {}", e);
                    break;
                }
            }
        });

        // Handle incoming messages
        while let Some(message) = ws_receiver.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Err(e) = self.handle_text_message(&text, &mut player_id, &tx).await {
                        error!("Error handling message: {}", e);
                        let error_msg = ServerMessage::Error {
                            message: "Internal server error".to_string(),
                        };
                        let _ = tx.send(error_msg);
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("Client disconnected: {:?}", player_id);
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        // Clean up on disconnect
        if let Some(id) = player_id {
            if let Err(e) = self.game_manager.remove_player(&id).await {
                error!("Failed to remove player {}: {}", id, e);
            }
        }

        // Stop the sender task
        sender_task.abort();

        Ok(())
    }

    async fn handle_text_message(
        &self,
        text: &str,
        player_id: &mut Option<String>,
        tx: &mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<()> {
        let client_msg: ClientMessage = serde_json::from_str(text)
            .map_err(|e| anyhow::anyhow!("Failed to parse message: {}", e))?;

        info!("Received: {:?}", client_msg);

        let response = match client_msg {
            ClientMessage::Connect { player_id: requested_id } => {
                self.handle_connect(requested_id, player_id).await?
            }
            ClientMessage::FindMatch => {
                self.handle_find_match(player_id, tx).await?
            }
            ClientMessage::PlayerMove { choice } => {
                self.handle_player_move(player_id, choice).await?
            }
        };

        if let Some(response) = response {
            tx.send(response)
                .map_err(|_| anyhow::anyhow!("Failed to send response"))?;
        }

        Ok(())
    }

    async fn handle_connect(
        &self,
        requested_id: Option<String>,
        player_id: &mut Option<String>,
    ) -> Result<Option<ServerMessage>> {
        let id = requested_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        *player_id = Some(id.clone());
        info!("Player connected with ID: {}", id);

        Ok(Some(ServerMessage::Connected { player_id: id }))
    }

    async fn handle_find_match(
        &self,
        player_id: &Option<String>,
        tx: &mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<Option<ServerMessage>> {
        if let Some(ref id) = player_id {
            let player = Arc::new(Player::new(id.clone(), tx.clone()));

            match self.game_manager.find_match(player).await {
                Ok(msg) => Ok(Some(msg)),
                Err(e) => {
                    error!("Find match error: {}", e);
                    Ok(Some(ServerMessage::Error {
                        message: "Failed to find match".to_string(),
                    }))
                }
            }
        } else {
            Ok(Some(ServerMessage::Error {
                message: "Not connected".to_string(),
            }))
        }
    }

    async fn handle_player_move(
        &self,
        player_id: &Option<String>,
        choice: crate::domain::GameChoice,
    ) -> Result<Option<ServerMessage>> {
        if let Some(ref id) = player_id {
            match self.game_manager.submit_move(id, choice).await {
                Ok(true) => Ok(None), // Move processed successfully
                Ok(false) => Ok(Some(ServerMessage::Error {
                    message: "Invalid move".to_string(),
                })),
                Err(e) => {
                    error!("Submit move error: {}", e);
                    Ok(Some(ServerMessage::Error {
                        message: "Failed to submit move".to_string(),
                    }))
                }
            }
        } else {
            Ok(Some(ServerMessage::Error {
                message: "Not connected".to_string(),
            }))
        }
    }
}