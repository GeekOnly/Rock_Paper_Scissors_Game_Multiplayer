use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

use crate::domain::{GameChoice, GameConfig, GameResult, GameStatus, Player, PlayerInfo, PlayerMove, ServerMessage};

pub struct GameRoom {
    pub id: String,
    pub players: Vec<Arc<Player>>,
    pub current_round: u32,
    pub config: GameConfig,
    pub scores: HashMap<String, u32>,
    pub moves: HashMap<String, PlayerMove>,
    pub status: GameStatus,
}

impl GameRoom {
    pub fn new(id: String, config: GameConfig) -> Self {
        Self {
            id,
            players: Vec::new(),
            current_round: 1,
            config,
            scores: HashMap::new(),
            moves: HashMap::new(),
            status: GameStatus::Waiting,
        }
    }

    pub fn add_player(&mut self, player: Arc<Player>) -> Result<bool> {
        if self.players.len() >= self.config.max_players {
            return Ok(false);
        }

        self.scores.insert(player.id.clone(), 0);
        self.players.push(player);

        if self.players.len() >= self.config.min_players {
            self.status = GameStatus::Playing;
        }

        Ok(true)
    }

    pub async fn start_game(&self) -> Result<()> {
        let message = ServerMessage::GameStart {
            room_id: self.id.clone(),
            players: self.players.iter().map(|p| PlayerInfo { id: p.id.clone() }).collect(),
            max_rounds: self.config.max_rounds,
        };

        self.broadcast_to_all(&message).await
    }

    pub fn submit_move(&mut self, player_id: &str, choice: GameChoice) -> Result<bool> {
        if self.status != GameStatus::Playing {
            return Ok(false);
        }

        if !self.players.iter().any(|p| p.id == player_id) {
            return Ok(false);
        }

        self.moves.insert(
            player_id.to_string(),
            PlayerMove {
                choice,
                timestamp: Utc::now(),
            },
        );

        // Check if all players have moved
        Ok(self.moves.len() == self.players.len())
    }

    pub async fn process_round(&mut self) -> Result<()> {
        let result = self.calculate_round_result()?;
        
        info!(
            "Round {}: {} vs {}",
            result.round,
            self.format_moves(&result.moves),
            self.format_winner(&result.winner)
        );

        // Update scores
        if let Some(ref winner_id) = result.winner {
            *self.scores.get_mut(winner_id).unwrap() += 1;
        }

        // Send round result
        let round_result = ServerMessage::RoundResult {
            round: result.round,
            winner: result.winner.clone(),
            moves: result.moves,
            scores: self.scores.clone(),
        };

        self.broadcast_to_all(&round_result).await?;

        // Check for game end
        if self.should_end_game() {
            self.end_game().await?;
        } else {
            self.next_round().await?;
        }

        Ok(())
    }

    fn calculate_round_result(&self) -> Result<GameResult> {
        let player_ids: Vec<String> = self.players.iter().map(|p| p.id.clone()).collect();
        
        if player_ids.len() != 2 {
            return Err(anyhow::anyhow!("Invalid number of players"));
        }

        let p1_move = &self.moves[&player_ids[0]];
        let p2_move = &self.moves[&player_ids[1]];

        let winner = if p1_move.choice == p2_move.choice {
            None // Draw
        } else if p1_move.choice.beats(&p2_move.choice) {
            Some(player_ids[0].clone())
        } else {
            Some(player_ids[1].clone())
        };

        let moves_map: HashMap<String, GameChoice> = self
            .moves
            .iter()
            .map(|(id, player_move)| (id.clone(), player_move.choice.clone()))
            .collect();

        Ok(GameResult {
            round: self.current_round,
            winner,
            moves: moves_map,
            scores: self.scores.clone(),
        })
    }

    fn should_end_game(&self) -> bool {
        let max_score = *self.scores.values().max().unwrap_or(&0);
        max_score >= 2 || self.current_round >= self.config.max_rounds
    }

    async fn next_round(&mut self) -> Result<()> {
        self.current_round += 1;
        self.moves.clear();

        let message = ServerMessage::NextRound {
            round: self.current_round,
        };

        self.broadcast_to_all(&message).await
    }

    async fn end_game(&mut self) -> Result<()> {
        self.status = GameStatus::Finished;

        let final_winner = self.determine_final_winner();

        let message = ServerMessage::GameEnd {
            winner: final_winner,
            final_scores: self.scores.clone(),
        };

        self.broadcast_to_all(&message).await
    }

    fn determine_final_winner(&self) -> Option<String> {
        let max_score = *self.scores.values().max().unwrap_or(&0);
        let winners: Vec<_> = self
            .scores
            .iter()
            .filter(|(_, &score)| score == max_score)
            .collect();

        if winners.len() == 1 {
            Some(winners[0].0.clone())
        } else {
            None // Tie game
        }
    }

    async fn broadcast_to_all(&self, message: &ServerMessage) -> Result<()> {
        for player in &self.players {
            if let Err(e) = player.send_message(message).await {
                warn!("Failed to send message to player {}: {}", player.id, e);
            }
        }
        Ok(())
    }

    pub async fn notify_player_left(&self, player_id: &str) -> Result<()> {
        let message = ServerMessage::PlayerLeft {
            player_id: player_id.to_string(),
        };
        self.broadcast_to_all(&message).await
    }

    fn format_moves(&self, moves: &HashMap<String, GameChoice>) -> String {
        moves
            .iter()
            .map(|(id, choice)| format!("{}: {:?}", id, choice))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_winner(&self, winner: &Option<String>) -> String {
        match winner {
            Some(id) => format!("Winner: {}", id),
            None => "Draw".to_string(),
        }
    }
}