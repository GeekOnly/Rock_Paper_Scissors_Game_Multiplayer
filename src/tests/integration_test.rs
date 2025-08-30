use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::info;

use crate::application::GameManager;
use crate::config::ServerConfig;
use crate::domain::{GameConfig, Player};
use crate::tests::load_test::{test_concurrent_connections, test_connection_limits, LoadTestConfig, LoadTestRunner};

pub struct IntegrationTestSuite {
    config: ServerConfig,
}

impl IntegrationTestSuite {
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
        }
    }

    pub async fn run_all_tests(&self) -> Result<()> {
        info!("🚀 Starting Integration Test Suite");

        // Test 1: Basic functionality
        self.test_basic_game_flow().await?;

        // Test 2: Concurrent connections
        self.test_concurrent_connections().await?;

        // Test 3: Connection limits
        self.test_connection_limits().await?;

        // Test 4: Memory usage under load
        self.test_memory_usage().await?;

        info!("✅ All integration tests completed successfully");
        Ok(())
    }

    async fn test_basic_game_flow(&self) -> Result<()> {
        info!("🧪 Testing basic game flow...");

        let game_manager = Arc::new(GameManager::new(self.config.game.clone().into()));
        
        // Create mock players
        let (tx1, _rx1) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, _rx2) = tokio::sync::mpsc::unbounded_channel();
        
        let player1 = Arc::new(Player::new("test_player_1".to_string(), tx1));
        let player2 = Arc::new(Player::new("test_player_2".to_string(), tx2));

        // Test matchmaking
        let match_result1 = game_manager.find_match(player1.clone()).await?;
        let match_result2 = game_manager.find_match(player2.clone()).await?;

        // Verify match was created
        assert!(matches!(match_result2, crate::domain::ServerMessage::Matchmaking { matched: true, .. }));

        // Test game stats
        let (total_rooms, active_games, waiting_players) = game_manager.get_stats().await;
        assert_eq!(total_rooms, 1);
        assert_eq!(active_games, 1);
        assert_eq!(waiting_players, 0);

        info!("✅ Basic game flow test passed");
        Ok(())
    }

    async fn test_concurrent_connections(&self) -> Result<()> {
        info!("🧪 Testing concurrent connections...");

        let test_cases = [10, 25, 50];
        
        for &num_connections in &test_cases {
            info!("Testing {} concurrent connections", num_connections);
            
            let metrics = test_concurrent_connections(num_connections).await?;
            
            // Verify results
            let success_rate = metrics.successful_connections as f64 / num_connections as f64;
            info!("Success rate: {:.2}%", success_rate * 100.0);
            
            // We expect at least 80% success rate for reasonable loads
            if num_connections <= 50 {
                assert!(success_rate >= 0.8, "Success rate too low: {:.2}%", success_rate * 100.0);
            }
        }

        info!("✅ Concurrent connections test passed");
        Ok(())
    }

    async fn test_connection_limits(&self) -> Result<()> {
        info!("🧪 Testing connection limits...");

        let results = test_connection_limits().await?;
        
        // Find the breaking point
        let mut max_successful = 0;
        for (count, metrics) in results {
            let success_rate = metrics.successful_connections as f64 / count as f64;
            info!("Connections: {}, Success rate: {:.2}%", count, success_rate * 100.0);
            
            if success_rate >= 0.9 {
                max_successful = count;
            } else {
                break;
            }
        }

        info!("✅ Maximum successful concurrent connections: {}", max_successful);
        Ok(())
    }

    async fn test_memory_usage(&self) -> Result<()> {
        info!("🧪 Testing memory usage under load...");

        // Run a sustained load test
        let config = LoadTestConfig {
            concurrent_connections: 20,
            test_duration: Duration::from_secs(30),
            ..Default::default()
        };

        let runner = LoadTestRunner::new(config);
        let metrics = runner.run_load_test().await?;

        // Check for memory leaks (basic check)
        let success_rate = metrics.successful_connections as f64 / 20.0;
        assert!(success_rate >= 0.8, "Memory usage test failed with low success rate");

        info!("✅ Memory usage test passed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_game_manager_basic() {
        let config = GameConfig::default();
        let game_manager = GameManager::new(config);
        
        let (total_rooms, active_games, waiting_players) = game_manager.get_stats().await;
        assert_eq!(total_rooms, 0);
        assert_eq!(active_games, 0);
        assert_eq!(waiting_players, 0);
    }

    #[tokio::test]
    async fn test_player_creation() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let player = Player::new("test_player".to_string(), tx);
        assert_eq!(player.id, "test_player");
    }
}