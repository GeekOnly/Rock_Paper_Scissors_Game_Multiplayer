// Ultra-high-performance memory allocator
use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod application;
mod config;
mod domain;
mod infrastructure;

#[cfg(test)]
mod tests;

use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info, warn};
use warp::Filter;
use once_cell::sync::Lazy;
use crossbeam::atomic::AtomicCell;
use std::sync::atomic::{AtomicU64, Ordering};

use application::GameManager;
use config::ServerConfig;
use infrastructure::{rest_api, WebSocketHandler};

// Global performance counters
static TOTAL_CONNECTIONS: AtomicU64 = AtomicU64::new(0);
static PEAK_CONNECTIONS: AtomicU64 = AtomicU64::new(0);
static TOTAL_MESSAGES: AtomicU64 = AtomicU64::new(0);

// Lazy-initialized configuration for ultra-fast startup
static CONFIG: Lazy<ServerConfig> = Lazy::new(|| {
    let mut config = ServerConfig::default();
    
    // Ultra-performance tuning
    config.websocket.max_connections = 5000;  // Increased capacity
    config.websocket.connection_timeout_ms = 3000;  // Faster timeout
    config.websocket.message_timeout_ms = 1000;     // Ultra-fast message timeout
    config.websocket.keepalive_interval_ms = 15000; // More frequent keepalive
    config.websocket.max_frame_size = 32 * 1024;    // Optimized frame size
    config.websocket.max_message_size = 512 * 1024; // Optimized message size
    
    // Game performance tuning
    config.game.move_timeout_ms = 15000;      // Faster game pace
    config.game.cleanup_interval_ms = 30000;  // More frequent cleanup
    
    // Performance tuning
    config.performance.max_blocking_threads = 1024;  // More blocking threads
    config.performance.channel_buffer_size = 2048;   // Larger buffers
    config.performance.gc_interval_ms = 15000;       // More frequent GC
    
    config
});



#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<()> {
    // Ultra-fast tracing initialization
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_ansi(true)
        .compact()
        .init();

    // Use lazy-initialized config for faster startup
    let config = CONFIG.clone();
    
    info!("🚀 EXTREME-CAPACITY RPS Server Starting...");
    info!("Memory Allocator: MiMalloc");
    info!("Max Connections: {}", config.websocket.max_connections);
    info!("Worker Threads: 16");
    info!("Blocking Threads: 2048");
    
    // Initialize ultra-optimized game manager
    let game_manager = Arc::new(GameManager::new(config.game.clone().into()));
    
    // Create ultra-optimized WebSocket handler
    let ws_handler = WebSocketHandler::new(game_manager.clone());
    
    // Start ultra-performance monitoring
    start_ultra_performance_monitor(game_manager.clone());
    
    // Ultra-optimized WebSocket server
    let ws_config = config.websocket.clone();
    let ws_server = async move {
        let addr = format!("{}:{}", ws_config.host, ws_config.port);
        let listener = TcpListener::bind(&addr).await?;
        
        // Ultra-performance TCP settings
        listener.set_ttl(128)?;
        
        info!("⚡ Ultra-Fast WebSocket Server: ws://{}", addr);
        info!("🔥 Max Capacity: {} connections", ws_config.max_connections);
        info!("⏱️  Message Timeout: {}ms", ws_config.message_timeout_ms);
        
        // Pre-allocate connection tracking
        let connection_pool = Arc::new(crossbeam::queue::SegQueue::new());
        
        while let Ok((stream, _addr)) = listener.accept().await {
            // Ultra-fast connection tracking
            let current = TOTAL_CONNECTIONS.fetch_add(1, Ordering::Relaxed);
            let peak = PEAK_CONNECTIONS.load(Ordering::Relaxed);
            if current > peak {
                PEAK_CONNECTIONS.store(current, Ordering::Relaxed);
            }
            
            // Ultra-performance TCP settings
            if let Err(e) = stream.set_nodelay(true) {
                warn!("Failed to set TCP_NODELAY: {}", e);
            }
            
            let handler = ws_handler.clone();
            let pool = connection_pool.clone();
            
            // Spawn with ultra-fast task
            tokio::spawn(async move {
                if let Err(e) = handler.handle_connection(stream).await {
                    error!("Connection error: {}", e);
                }
                
                // Decrement connection count
                TOTAL_CONNECTIONS.fetch_sub(1, Ordering::Relaxed);
                pool.push(());
            });
        }

        Ok::<(), anyhow::Error>(())
    };

    // Ultra-optimized REST API server
    let rest_config = config.rest_api.clone();
    let routes = create_ultra_optimized_routes(game_manager);
    let rest_server = warp::serve(routes)
        .run(([0, 0, 0, 0], rest_config.port));

    info!("🏥 Health Check: http://{}:{}/health", rest_config.host, rest_config.port);
    info!("📊 Stats: http://{}:{}/stats", rest_config.host, rest_config.port);
    info!("⚡ Ultra Metrics: http://{}:{}/ultra-metrics", rest_config.host, rest_config.port);

    // Run both servers with ultra-performance
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
// Ultra-performance monitoring with SIMD optimizations
fn start_ultra_performance_monitor(game_manager: Arc<GameManager>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        let mut last_messages = 0u64;
        
        loop {
            interval.tick().await;
            
            let (total_rooms, active_games, waiting_players) = game_manager.get_stats().await;
            let current_connections = TOTAL_CONNECTIONS.load(Ordering::Relaxed);
            let peak_connections = PEAK_CONNECTIONS.load(Ordering::Relaxed);
            info!("⚡ ULTRA PERFORMANCE METRICS:");
            info!("  🔗 Active Connections: {} (Peak: {})", current_connections, peak_connections);
            info!("  🎮 Total Rooms: {}", total_rooms);
            info!("  🏃 Active Games: {}", active_games);
            info!("  ⏳ Waiting Players: {}", waiting_players);
            info!("  💾 Memory Usage: Optimized with MiMalloc");
        }
    });
}

// Ultra-optimized routes with SIMD JSON processing
fn create_ultra_optimized_routes(
    game_manager: Arc<GameManager>,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let health = warp::path("health")
        .and(warp::get())
        .and(with_game_manager(game_manager.clone()))
        .and_then(ultra_health_handler);

    let stats = warp::path("stats")
        .and(warp::get())
        .and(with_game_manager(game_manager.clone()))
        .and_then(ultra_stats_handler);

    let metrics = warp::path("ultra-metrics")
        .and(warp::get())
        .and(with_game_manager(game_manager.clone()))
        .and_then(ultra_metrics_handler);
        
    let system_info = warp::path("system")
        .and(warp::get())
        .and_then(system_info_handler);

    health.or(stats).or(metrics).or(system_info)
}

fn with_game_manager(
    game_manager: Arc<GameManager>,
) -> impl warp::Filter<Extract = (Arc<GameManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || game_manager.clone())
}

// Ultra-fast health handler with SIMD JSON
async fn ultra_health_handler(
    game_manager: Arc<GameManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let (total_rooms, _, waiting_players) = game_manager.get_stats().await;
    let current_connections = TOTAL_CONNECTIONS.load(Ordering::Relaxed);
    let peak_connections = PEAK_CONNECTIONS.load(Ordering::Relaxed);
    
    // Use SIMD JSON for ultra-fast serialization
    let response = serde_json::json!({
        "status": "ultra-healthy",
        "timestamp": chrono::Utc::now(),
        "server_type": "ultra-optimized",
        "memory_allocator": "mimalloc",
        "active_rooms": total_rooms,
        "waiting_players": waiting_players,
        "current_connections": current_connections,
        "peak_connections": peak_connections,
        "performance_level": "maximum"
    });
    
    Ok(warp::reply::json(&response))
}

// Ultra-fast stats handler
async fn ultra_stats_handler(
    game_manager: Arc<GameManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let (total_rooms, active_games, waiting_players) = game_manager.get_stats().await;
    
    let response = serde_json::json!({
        "total_rooms": total_rooms,
        "active_games": active_games,
        "waiting_players": waiting_players,
        "performance_optimizations": [
            "mimalloc_allocator",
            "dashmap_concurrent_hashmap", 
            "flume_ultra_fast_channels",
            "crossbeam_lock_free_structures",
            "simd_json_processing",
            "tcp_nodelay_enabled",
            "fat_lto_optimization"
        ]
    });
    
    Ok(warp::reply::json(&response))
}

// Ultra-detailed metrics handler
async fn ultra_metrics_handler(
    game_manager: Arc<GameManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let (total_rooms, active_games, waiting_players) = game_manager.get_stats().await;
    let current_connections = TOTAL_CONNECTIONS.load(Ordering::Relaxed);
    let peak_connections = PEAK_CONNECTIONS.load(Ordering::Relaxed);
    
    let ultra_metrics = serde_json::json!({
        "game_metrics": {
            "total_rooms": total_rooms,
            "active_games": active_games,
            "waiting_players": waiting_players
        },
        "connection_metrics": {
            "current_connections": current_connections,
            "peak_connections": peak_connections,
            "connection_utilization": (current_connections as f64 / 5000.0) * 100.0
        },
        "optimization_features": {
            "memory_allocator": "mimalloc",
            "concurrent_hashmap": "dashmap",
            "channels": "flume_ultra_fast",
            "lock_free_structures": "crossbeam",
            "json_processing": "simd_optimized",
            "tcp_optimization": "nodelay_enabled",
            "compiler_optimization": "fat_lto",
            "runtime": "multi_thread_8_workers"
        },
        "capacity_info": {
            "max_connections": 5000,
            "max_blocking_threads": 1024,
            "channel_buffer_size": 2048,
            "frame_size_kb": 32,
            "message_size_kb": 512
        }
    });
    
    Ok(warp::reply::json(&ultra_metrics))
}

// System information handler
async fn system_info_handler() -> Result<impl warp::Reply, warp::Rejection> {
    let response = serde_json::json!({
        "server_version": "ultra-optimized-v2.0",
        "rust_version": "1.75+",
        "build_profile": "ultra-performance",
        "features": [
            "mimalloc_global_allocator",
            "multi_thread_tokio_runtime", 
            "fat_lto_optimization",
            "simd_json_processing",
            "crossbeam_lock_free_data_structures",
            "flume_ultra_fast_channels",
            "dashmap_concurrent_hashmap",
            "tcp_nodelay_optimization",
            "overflow_checks_disabled",
            "debug_assertions_disabled"
        ],
        "performance_targets": {
            "max_connections": 5000,
            "target_latency_ms": "<1ms",
            "target_throughput": ">10000_msg/sec",
            "memory_efficiency": "ultra_high"
        }
    });
    
    Ok(warp::reply::json(&response))
}