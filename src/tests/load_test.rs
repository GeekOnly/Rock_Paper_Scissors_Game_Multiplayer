use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::Barrier;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{error, info, warn};

use crate::domain::{ClientMessage, GameChoice, ServerMessage};

#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    pub server_url: String,
    pub concurrent_connections: usize,
    pub test_duration: Duration,
    pub connection_timeout: Duration,
    pub message_timeout: Duration,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://127.0.0.1:8080".to_string(),
            concurrent_connections: 100,
            test_duration: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(5),
            message_timeout: Duration::from_secs(10),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadTestMetrics {
    pub successful_connections: u32,
    pub failed_connections: u32,
    pub successful_matches: u32,
    pub failed_matches: u32,
    pub completed_games: u32,
    pub total_messages_sent: u32,
    pub total_messages_received: u32,
    pub average_connection_time: Duration,
    pub average_match_time: Duration,
    pub errors: Vec<String>,
}

impl Default for LoadTestMetrics {
    fn default() -> Self {
        Self {
            successful_connections: 0,
            failed_connections: 0,
            successful_matches: 0,
            failed_matches: 0,
            completed_games: 0,
            total_messages_sent: 0,
            total_messages_received: 0,
            average_connection_time: Duration::ZERO,
            average_match_time: Duration::ZERO,
            errors: Vec::new(),
        }
    }
}

pub struct LoadTestRunner {
    config: LoadTestConfig,
    metrics: Arc<LoadTestMetrics>,
    // Atomic counters for thread-safe metrics
    successful_connections: Arc<AtomicU32>,
    failed_connections: Arc<AtomicU32>,
    successful_matches: Arc<AtomicU32>,
    failed_matches: Arc<AtomicU32>,
    completed_games: Arc<AtomicU32>,
    messages_sent: Arc<AtomicU32>,
    messages_received: Arc<AtomicU32>,
}

impl LoadTestRunner {
    pub fn new(config: LoadTestConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(LoadTestMetrics::default()),
            successful_connections: Arc::new(AtomicU32::new(0)),
            failed_connections: Arc::new(AtomicU32::new(0)),
            successful_matches: Arc::new(AtomicU32::new(0)),
            failed_matches: Arc::new(AtomicU32::new(0)),
            completed_games: Arc::new(AtomicU32::new(0)),
            messages_sent: Arc::new(AtomicU32::new(0)),
            messages_received: Arc::new(AtomicU32::new(0)),
        }
    }

    pub async fn run_load_test(&self) -> Result<LoadTestMetrics> {
        info!("Starting load test with {} concurrent connections", self.config.concurrent_connections);
        
        let start_time = Instant::now();
        let barrier = Arc::new(Barrier::new(self.config.concurrent_connections));
        
        let mut handles = Vec::new();

        // Spawn concurrent client tasks
        for i in 0..self.config.concurrent_connections {
            let config = self.config.clone();
            let barrier = barrier.clone();
            let successful_connections = self.successful_connections.clone();
            let failed_connections = self.failed_connections.clone();
            let successful_matches = self.successful_matches.clone();
            let failed_matches = self.failed_matches.clone();
            let completed_games = self.completed_games.clone();
            let messages_sent = self.messages_sent.clone();
            let messages_received = self.messages_received.clone();

            let handle = tokio::spawn(async move {
                // Wait for all clients to be ready
                barrier.wait().await;
                
                let client_id = format!("load_test_client_{}", i);
                match Self::run_client_session(
                    client_id,
                    config,
                    successful_connections,
                    failed_connections,
                    successful_matches,
                    failed_matches,
                    completed_games,
                    messages_sent,
                    messages_received,
                ).await {
                    Ok(_) => info!("Client {} completed successfully", i),
                    Err(e) => error!("Client {} failed: {}", i, e),
                }
            });
            
            handles.push(handle);
        }

        // Wait for all clients to complete or timeout
        let test_timeout = timeout(self.config.test_duration, async {
            for handle in handles {
                let _ = handle.await;
            }
        }).await;

        if test_timeout.is_err() {
            warn!("Load test timed out after {:?}", self.config.test_duration);
        }

        let total_time = start_time.elapsed();
        
        // Collect final metrics
        let final_metrics = LoadTestMetrics {
            successful_connections: self.successful_connections.load(Ordering::Relaxed),
            failed_connections: self.failed_connections.load(Ordering::Relaxed),
            successful_matches: self.successful_matches.load(Ordering::Relaxed),
            failed_matches: self.failed_matches.load(Ordering::Relaxed),
            completed_games: self.completed_games.load(Ordering::Relaxed),
            total_messages_sent: self.messages_sent.load(Ordering::Relaxed),
            total_messages_received: self.messages_received.load(Ordering::Relaxed),
            average_connection_time: total_time / self.config.concurrent_connections as u32,
            average_match_time: Duration::ZERO, // TODO: Calculate properly
            errors: Vec::new(), // TODO: Collect errors
        };

        info!("Load test completed in {:?}", total_time);
        info!("Results: {:?}", final_metrics);

        Ok(final_metrics)
    }

    async fn run_client_session(
        client_id: String,
        config: LoadTestConfig,
        successful_connections: Arc<AtomicU32>,
        failed_connections: Arc<AtomicU32>,
        successful_matches: Arc<AtomicU32>,
        failed_matches: Arc<AtomicU32>,
        completed_games: Arc<AtomicU32>,
        messages_sent: Arc<AtomicU32>,
        messages_received: Arc<AtomicU32>,
    ) -> Result<()> {
        // Connect to server
        let connection_start = Instant::now();
        let ws_stream = match timeout(config.connection_timeout, connect_async(&config.server_url)).await {
            Ok(Ok((ws_stream, _))) => {
                successful_connections.fetch_add(1, Ordering::Relaxed);
                ws_stream
            }
            Ok(Err(e)) => {
                failed_connections.fetch_add(1, Ordering::Relaxed);
                return Err(anyhow::anyhow!("Connection failed: {}", e));
            }
            Err(_) => {
                failed_connections.fetch_add(1, Ordering::Relaxed);
                return Err(anyhow::anyhow!("Connection timeout"));
            }
        };

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        // Send connect message
        let connect_msg = ClientMessage::Connect {
            player_id: Some(client_id.clone()),
        };
        
        Self::send_message(&mut ws_sender, &connect_msg, &messages_sent).await?;
        
        // Wait for connected response
        let _connected_msg = Self::receive_message(&mut ws_receiver, &messages_received, &config).await?;
        
        // Send find match
        let find_match_msg = ClientMessage::FindMatch;
        Self::send_message(&mut ws_sender, &find_match_msg, &messages_sent).await?;
        
        // Wait for matchmaking response
        let match_start = Instant::now();
        loop {
            let msg = Self::receive_message(&mut ws_receiver, &messages_received, &config).await?;
            
            match msg {
                ServerMessage::Matchmaking { matched: true, .. } => {
                    successful_matches.fetch_add(1, Ordering::Relaxed);
                    break;
                }
                ServerMessage::Matchmaking { matched: false, .. } => {
                    // Still waiting for match
                    continue;
                }
                ServerMessage::GameStart { .. } => {
                    // Game started
                    break;
                }
                _ => continue,
            }
        }
        
        // Play the game
        Self::play_game(&mut ws_sender, &mut ws_receiver, &config, &messages_sent, &messages_received).await?;
        
        completed_games.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }

    async fn play_game(
        ws_sender: &mut futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        ws_receiver: &mut futures_util::stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        config: &LoadTestConfig,
        messages_sent: &Arc<AtomicU32>,
        messages_received: &Arc<AtomicU32>,
    ) -> Result<()> {
        let moves = [GameChoice::Rock, GameChoice::Paper, GameChoice::Scissors];
        let mut round = 0;
        
        loop {
            // Make a random move
            let choice = moves[round % moves.len()].clone();
            let move_msg = ClientMessage::PlayerMove { choice };
            
            Self::send_message(ws_sender, &move_msg, messages_sent).await?;
            
            // Wait for round result or game end
            loop {
                let msg = Self::receive_message(ws_receiver, messages_received, config).await?;
                
                match msg {
                    ServerMessage::RoundResult { .. } => {
                        // Round completed
                        break;
                    }
                    ServerMessage::NextRound { .. } => {
                        // Next round starting
                        round += 1;
                        break;
                    }
                    ServerMessage::GameEnd { .. } => {
                        // Game finished
                        return Ok(());
                    }
                    _ => continue,
                }
            }
            
            if round >= 10 {
                // Safety limit
                break;
            }
        }
        
        Ok(())
    }

    async fn send_message(
        ws_sender: &mut futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        message: &ClientMessage,
        messages_sent: &Arc<AtomicU32>,
    ) -> Result<()> {
        let json = serde_json::to_string(message)?;
        ws_sender.send(Message::Text(json)).await?;
        messages_sent.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    async fn receive_message(
        ws_receiver: &mut futures_util::stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        messages_received: &Arc<AtomicU32>,
        config: &LoadTestConfig,
    ) -> Result<ServerMessage> {
        let msg = timeout(config.message_timeout, ws_receiver.next()).await
            .map_err(|_| anyhow::anyhow!("Message receive timeout"))?
            .ok_or_else(|| anyhow::anyhow!("Connection closed"))?
            .map_err(|e| anyhow::anyhow!("WebSocket error: {}", e))?;
        
        match msg {
            Message::Text(text) => {
                messages_received.fetch_add(1, Ordering::Relaxed);
                let server_msg: ServerMessage = serde_json::from_str(&text)?;
                Ok(server_msg)
            }
            _ => Err(anyhow::anyhow!("Unexpected message type")),
        }
    }
}

// Convenience functions for different test scenarios
pub async fn test_concurrent_connections(num_connections: usize) -> Result<LoadTestMetrics> {
    let config = LoadTestConfig {
        concurrent_connections: num_connections,
        test_duration: Duration::from_secs(60),
        ..Default::default()
    };
    
    let runner = LoadTestRunner::new(config);
    runner.run_load_test().await
}

pub async fn test_sustained_load(duration_secs: u64) -> Result<LoadTestMetrics> {
    let config = LoadTestConfig {
        concurrent_connections: 50,
        test_duration: Duration::from_secs(duration_secs),
        ..Default::default()
    };
    
    let runner = LoadTestRunner::new(config);
    runner.run_load_test().await
}

pub async fn test_connection_limits() -> Result<Vec<(usize, LoadTestMetrics)>> {
    let connection_counts = [10, 25, 50, 100, 200, 500, 1000];
    let mut results = Vec::new();
    
    for &count in &connection_counts {
        info!("Testing {} concurrent connections", count);
        
        let config = LoadTestConfig {
            concurrent_connections: count,
            test_duration: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            ..Default::default()
        };
        
        let runner = LoadTestRunner::new(config);
        match runner.run_load_test().await {
            Ok(metrics) => {
                info!("✅ {} connections: {} successful, {} failed", 
                      count, metrics.successful_connections, metrics.failed_connections);
                results.push((count, metrics));
            }
            Err(e) => {
                error!("❌ {} connections failed: {}", count, e);
                break;
            }
        }
        
        // Wait between tests
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    
    Ok(results)
}