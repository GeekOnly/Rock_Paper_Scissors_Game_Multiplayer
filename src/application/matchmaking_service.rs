use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::info;
use uuid::Uuid;

use crate::domain::{GameChoice, GameConfig, Player, ServerMessage};
use super::game_service::GameRoom;

pub struct GameManager {
    rooms: Arc<RwLock<HashMap<String, Arc<Mutex<GameRoom>>>>>,
    waiting_queue: Arc<Mutex<Vec<Arc<Player>>>>,
    player_rooms: Arc<RwLock<HashMap<String, String>>>, // playerId -> roomId
    config: GameConfig,
}

impl GameManager {
    pub fn new(config: GameConfig) -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            waiting_queue: Arc::new(Mutex::new(Vec::new())),
            player_rooms: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn find_match(&self, player: Arc<Player>) -> Result<ServerMessage> {
        let waiting_player = {
            let mut queue = self.waiting_queue.lock().await;
            queue.pop()
        };

        if let Some(waiting_player) = waiting_player {
            self.create_match(waiting_player, player).await
        } else {
            self.add_to_queue(player).await
        }
    }

    async fn create_match(&self, player1: Arc<Player>, player2: Arc<Player>) -> Result<ServerMessage> {
        let room_id = Uuid::new_v4().to_string();
        let mut room = GameRoom::new(room_id.clone(), self.config.clone());

        room.add_player(player1.clone())?;
        room.add_player(player2.clone())?;

        let room_arc = Arc::new(Mutex::new(room));

        // Store room and player mappings
        {
            let mut rooms = self.rooms.write().await;
            rooms.insert(room_id.clone(), room_arc.clone());
        }
        {
            let mut player_rooms = self.player_rooms.write().await;
            player_rooms.insert(player1.id.clone(), room_id.clone());
            player_rooms.insert(player2.id.clone(), room_id.clone());
        }

        // Start the game
        {
            let room = room_arc.lock().await;
            room.start_game().await?;
        }

        info!("Match created: {} vs {}", player1.id, player2.id);

        Ok(ServerMessage::Matchmaking {
            matched: true,
            waiting: None,
            room_id: Some(room_id),
        })
    }

    async fn add_to_queue(&self, player: Arc<Player>) -> Result<ServerMessage> {
        let mut queue = self.waiting_queue.lock().await;
        queue.push(player);

        Ok(ServerMessage::Matchmaking {
            matched: false,
            waiting: Some(true),
            room_id: None,
        })
    }

    pub async fn submit_move(&self, player_id: &str, choice: GameChoice) -> Result<bool> {
        let room_arc = self.get_player_room(player_id).await;

        if let Some(room_arc) = room_arc {
            let should_process = {
                let mut room = room_arc.lock().await;
                room.submit_move(player_id, choice)?
            };

            if should_process {
                let mut room = room_arc.lock().await;
                room.process_round().await?;
            }

            return Ok(true);
        }

        Ok(false)
    }

    pub async fn remove_player(&self, player_id: &str) -> Result<()> {
        // Remove from waiting queue
        {
            let mut queue = self.waiting_queue.lock().await;
            queue.retain(|p| p.id != player_id);
        }

        // Remove from room if exists
        let room_id = {
            let mut player_rooms = self.player_rooms.write().await;
            player_rooms.remove(player_id)
        };

        if let Some(room_id) = room_id {
            let room_arc = {
                let mut rooms = self.rooms.write().await;
                rooms.remove(&room_id)
            };

            if let Some(room_arc) = room_arc {
                let room = room_arc.lock().await;
                room.notify_player_left(player_id).await?;
            }
        }

        Ok(())
    }

    pub async fn get_stats(&self) -> (usize, usize, usize) {
        let rooms = self.rooms.read().await;
        let queue = self.waiting_queue.lock().await;

        let total_rooms = rooms.len();
        let mut active_games = 0;
        for room_arc in rooms.values() {
            let room = room_arc.lock().await;
            if room.status == crate::domain::GameStatus::Playing {
                active_games += 1;
            }
        }
        let waiting_players = queue.len();

        (total_rooms, active_games, waiting_players)
    }

    async fn get_player_room(&self, player_id: &str) -> Option<Arc<Mutex<GameRoom>>> {
        let room_id = {
            let player_rooms = self.player_rooms.read().await;
            player_rooms.get(player_id).cloned()
        };

        if let Some(room_id) = room_id {
            let rooms = self.rooms.read().await;
            rooms.get(&room_id).cloned()
        } else {
            None
        }
    }
}