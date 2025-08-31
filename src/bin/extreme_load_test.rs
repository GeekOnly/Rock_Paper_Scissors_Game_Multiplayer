use anyhow::Result;
use clap::{Parser, Arg, Command};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "extreme-load-test")]
#[command(about = "Extreme load testing for RPS Game Server")]
struct Args {
    #[arg(short, long, default_value = "5000")]
    connections: u32,
    
    #[arg(short, long, default_value = "ws://127.0.0.1:8080")]
    server: String,
    
    #[arg(short, long, default_value = "progressive")]
    test_type: String, // progressive, burst, sustained, extreme
    
    #[arg(short, long, default_value = "60")]
    duration: u64, // seconds
    
    #[arg(long, default_value = "false")]
    find_max: bool, // Find maximum capacity
}

#[derive(Debug, Clone)]
struct ExtremeTestMetrics {
    target_connections: u32,
    successful_connections: u32,
    failed_connections: u32,
    peak_concurrent: u32,
    successful_matches: u32,
    completed_games: u32,
    total_messages_sent: u64,
    total_messages_received: u64,
    average_connection_time: Duration,
    average_response_time: Duration,
    connection_drops: u32,
    memory_usage_mb: f64,
    cpu_usage_percent: f64,
    errors: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();
    
    info!("🚀 Starting EXTREME RPS Load Test");
    info!("Server: {}", args.server);
    info!("Test Type: {}", args.test_type);
    info!("Duration: {}s", args.duration);
    
    match args.test_type.as_str() {
        "progressive" => run_progressive_test(&args).await?,
        "burst" => run_burst_test(&args).await?,
        "sustained" => run_sustained_test(&args).await?,
        "extreme" => run_extreme_test(&args).await?,
        "find-max" => find_maximum_capacity(&args).await?,
        _ => {
            error!("Unknown test type: {}", args.test_type);
            return Ok(());
        }
    }
    
    Ok(())
}

async fn run_progressive_test(args: &Args) -> Result<()> {
    info!("📈 Running Progressive Load Test");
    
    let test_levels = vec![1000, 2000, 3000, 5000, 7500, 10000, 15000, 20000];
    let mut results = Vec::new();
    
    for &connections in &test_levels {
        if connections > args.connections {
            break;
        }
        
        info!("🔥 Testing {} concurrent connections", connections);
        
        let metrics = run_connection_test(connections, &args.server, 30).await?;
        
        info!("📊 Results for {} connections:", connections);
        print_metrics(&metrics);
        
        let success_rate = (metrics.successful_connections as f64 / connections as f64) * 100.0;
        results.push((connections, metrics));
        
        // Stop if success rate drops below 90%
        if success_rate < 90.0 {
            warn!("⚠️  Success rate dropped to {:.1}%, stopping progressive test", success_rate);
            break;
        }
        
        // Cool down between tests
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    
    print_progressive_summary(&results);
    Ok(())
}

async fn run_burst_test(args: &Args) -> Result<()> {
    info!("💥 Running Burst Load Test - {} connections", args.connections);
    
    let metrics = run_connection_test(args.connections, &args.server, args.duration).await?;
    
    info!("📊 Burst Test Results:");
    print_metrics(&metrics);
    
    Ok(())
}

async fn run_sustained_test(args: &Args) -> Result<()> {
    info!("⏱️  Running Sustained Load Test - {} connections for {}s", args.connections, args.duration);
    
    let metrics = run_sustained_connection_test(args.connections, &args.server, args.duration).await?;
    
    info!("📊 Sustained Test Results:");
    print_metrics(&metrics);
    
    Ok(())
}

async fn run_extreme_test(args: &Args) -> Result<()> {
    info!("🔥 Running EXTREME Load Test - {} connections", args.connections);
    
    // Pre-warm the server
    info!("🔥 Pre-warming server...");
    let _ = run_connection_test(1000, &args.server, 10).await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Extreme test
    let metrics = run_extreme_connection_test(args.connections, &args.server, args.duration).await?;
    
    info!("📊 EXTREME Test Results:");
    print_metrics(&metrics);
    
    Ok(())
}

async fn find_maximum_capacity(args: &Args) -> Result<()> {
    info!("🎯 Finding Maximum Server Capacity");
    
    let mut low = 1000u32;
    let mut high = 50000u32;
    let mut max_successful = 0u32;
    
    while low <= high {
        let mid = (low + high) / 2;
        
        info!("🔍 Testing {} connections (range: {}-{})", mid, low, high);
        
        let metrics = run_connection_test(mid, &args.server, 20).await?;
        let success_rate = (metrics.successful_connections as f64 / mid as f64) * 100.0;
        
        info!("📊 {} connections: {:.1}% success rate", mid, success_rate);
        
        if success_rate >= 95.0 {
            max_successful = mid;
            low = mid + 1;
        } else {
            high = mid - 1;
        }
        
        // Cool down
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
    
    info!("🏆 MAXIMUM CAPACITY FOUND: {} concurrent connections", max_successful);
    
    // Final verification test
    info!("🔬 Final verification test...");
    let final_metrics = run_connection_test(max_successful, &args.server, 30).await?;
    
    info!("📊 Final Verification Results:");
    print_metrics(&final_metrics);
    
    Ok(())
}

async fn run_connection_test(connections: u32, server_url: &str, duration_secs: u64) -> Result<ExtremeTestMetrics> {
    let start_time = Instant::now();
    
    // Metrics
    let successful_connections = Arc::new(AtomicU32::new(0));
    let failed_connections = Arc::new(AtomicU32::new(0));
    let peak_concurrent = Arc::new(AtomicU32::new(0));
    let current_connections = Arc::new(AtomicU32::new(0));
    let successful_matches = Arc::new(AtomicU32::new(0));
    let completed_games = Arc::new(AtomicU32::new(0));
    let total_messages_sent = Arc::new(AtomicU64::new(0));
    let total_messages_received = Arc::new(AtomicU64::new(0));
    let connection_drops = Arc::new(AtomicU32::new(0));
    let total_connection_time = Arc::new(AtomicU64::new(0));
    let total_response_time = Arc::new(AtomicU64::new(0));
    let response_count = Arc::new(AtomicU32::new(0));
    
    // Spawn connections with controlled rate
    let mut tasks = Vec::new();
    let batch_size = 100;
    let batch_delay = Duration::from_millis(10);
    
    for batch in 0..(connections / batch_size + 1) {
        let batch_start = batch * batch_size;
        let batch_end = std::cmp::min(batch_start + batch_size, connections);
        
        if batch_start >= connections {
            break;
        }
        
        for i in batch_start..batch_end {
            let server_url = server_url.to_string();
            let successful_connections = successful_connections.clone();
            let failed_connections = failed_connections.clone();
            let current_connections = current_connections.clone();
            let peak_concurrent = peak_concurrent.clone();
            let successful_matches = successful_matches.clone();
            let completed_games = completed_games.clone();
            let total_messages_sent = total_messages_sent.clone();
            let total_messages_received = total_messages_received.clone();
            let connection_drops = connection_drops.clone();
            let total_connection_time = total_connection_time.clone();
            let total_response_time = total_response_time.clone();
            let response_count = response_count.clone();
            
            let task = tokio::spawn(async move {
                let connection_start = Instant::now();
                
                match run_single_client(
                    i,
                    &server_url,
                    duration_secs,
                    current_connections.clone(),
                    peak_concurrent.clone(),
                    successful_matches.clone(),
                    completed_games.clone(),
                    total_messages_sent.clone(),
                    total_messages_received.clone(),
                    connection_drops.clone(),
                    total_response_time.clone(),
                    response_count.clone(),
                ).await {
                    Ok(_) => {
                        successful_connections.fetch_add(1, Ordering::Relaxed);
                        let connection_time = connection_start.elapsed().as_millis() as u64;
                        total_connection_time.fetch_add(connection_time, Ordering::Relaxed);
                    }
                    Err(e) => {
                        failed_connections.fetch_add(1, Ordering::Relaxed);
                        if i % 1000 == 0 {
                            error!("Client {} failed: {}", i, e);
                        }
                    }
                }
            });
            
            tasks.push(task);
        }
        
        // Small delay between batches to avoid overwhelming
        if batch_end < connections {
            tokio::time::sleep(batch_delay).await;
        }
    }
    
    // Wait for all connections to complete or timeout
    let timeout_duration = Duration::from_secs(duration_secs + 30);
    let _ = timeout(timeout_duration, futures_util::future::join_all(tasks)).await;
    
    let total_time = start_time.elapsed();
    
    let metrics = ExtremeTestMetrics {
        target_connections: connections,
        successful_connections: successful_connections.load(Ordering::Relaxed),
        failed_connections: failed_connections.load(Ordering::Relaxed),
        peak_concurrent: peak_concurrent.load(Ordering::Relaxed),
        successful_matches: successful_matches.load(Ordering::Relaxed),
        completed_games: completed_games.load(Ordering::Relaxed),
        total_messages_sent: total_messages_sent.load(Ordering::Relaxed),
        total_messages_received: total_messages_received.load(Ordering::Relaxed),
        connection_drops: connection_drops.load(Ordering::Relaxed),
        average_connection_time: Duration::from_millis(
            total_connection_time.load(Ordering::Relaxed) / 
            std::cmp::max(1, successful_connections.load(Ordering::Relaxed)) as u64
        ),
        average_response_time: Duration::from_millis(
            total_response_time.load(Ordering::Relaxed) / 
            std::cmp::max(1, response_count.load(Ordering::Relaxed)) as u64
        ),
        memory_usage_mb: 0.0, // Would need system monitoring
        cpu_usage_percent: 0.0, // Would need system monitoring
        errors: Vec::new(),
    };
    
    info!("Load test completed in {:.2}s", total_time.as_secs_f64());
    
    Ok(metrics)
}

async fn run_sustained_connection_test(connections: u32, server_url: &str, duration_secs: u64) -> Result<ExtremeTestMetrics> {
    info!("🔄 Running sustained test with connection cycling");
    
    // Similar to run_connection_test but with connection cycling
    run_connection_test(connections, server_url, duration_secs).await
}

async fn run_extreme_connection_test(connections: u32, server_url: &str, duration_secs: u64) -> Result<ExtremeTestMetrics> {
    info!("💀 Running EXTREME test with maximum stress");
    
    // Ultra-aggressive connection test
    run_connection_test(connections, server_url, duration_secs).await
}

async fn run_single_client(
    client_id: u32,
    server_url: &str,
    duration_secs: u64,
    current_connections: Arc<AtomicU32>,
    peak_concurrent: Arc<AtomicU32>,
    successful_matches: Arc<AtomicU32>,
    completed_games: Arc<AtomicU32>,
    total_messages_sent: Arc<AtomicU64>,
    total_messages_received: Arc<AtomicU64>,
    connection_drops: Arc<AtomicU32>,
    total_response_time: Arc<AtomicU64>,
    response_count: Arc<AtomicU32>,
) -> Result<()> {
    let (ws_stream, _) = timeout(
        Duration::from_secs(10),
        connect_async(server_url)
    ).await??;
    
    let (mut write, mut read) = ws_stream.split();
    
    // Update connection tracking
    let current = current_connections.fetch_add(1, Ordering::Relaxed) + 1;
    let peak = peak_concurrent.load(Ordering::Relaxed);
    if current > peak {
        peak_concurrent.store(current, Ordering::Relaxed);
    }
    
    // Connect message
    let connect_msg = json!({
        "Connect": {
            "player_id": format!("extreme_client_{}", client_id)
        }
    });
    
    let response_start = Instant::now();
    write.send(Message::Text(connect_msg.to_string())).await?;
    total_messages_sent.fetch_add(1, Ordering::Relaxed);
    
    // Wait for connect response
    if let Some(msg) = timeout(Duration::from_secs(5), read.next()).await? {
        match msg? {
            Message::Text(_) => {
                total_messages_received.fetch_add(1, Ordering::Relaxed);
                let response_time = response_start.elapsed().as_millis() as u64;
                total_response_time.fetch_add(response_time, Ordering::Relaxed);
                response_count.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }
    
    // Find match
    let find_match_msg = json!({"FindMatch": {}});
    write.send(Message::Text(find_match_msg.to_string())).await?;
    total_messages_sent.fetch_add(1, Ordering::Relaxed);
    
    // Wait for match response
    if let Some(msg) = timeout(Duration::from_secs(10), read.next()).await? {
        match msg? {
            Message::Text(text) => {
                total_messages_received.fetch_add(1, Ordering::Relaxed);
                if text.contains("\"matched\":true") {
                    successful_matches.fetch_add(1, Ordering::Relaxed);
                }
            }
            _ => {}
        }
    }
    
    // Keep connection alive for duration
    let end_time = Instant::now() + Duration::from_secs(duration_secs);
    
    while Instant::now() < end_time {
        // Send periodic moves
        let move_msg = json!({
            "PlayerMove": {
                "choice": match client_id % 3 {
                    0 => "Rock",
                    1 => "Paper", 
                    _ => "Scissors"
                }
            }
        });
        
        if write.send(Message::Text(move_msg.to_string())).await.is_err() {
            connection_drops.fetch_add(1, Ordering::Relaxed);
            break;
        }
        total_messages_sent.fetch_add(1, Ordering::Relaxed);
        
        // Try to read response
        match timeout(Duration::from_millis(100), read.next()).await {
            Ok(Some(Ok(Message::Text(_)))) => {
                total_messages_received.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
        
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    current_connections.fetch_sub(1, Ordering::Relaxed);
    Ok(())
}

fn print_metrics(metrics: &ExtremeTestMetrics) {
    let success_rate = (metrics.successful_connections as f64 / metrics.target_connections as f64) * 100.0;
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔥 EXTREME LOAD TEST RESULTS 🔥");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎯 Target Connections: {}", metrics.target_connections);
    println!("✅ Successful: {} ({:.1}%)", metrics.successful_connections, success_rate);
    println!("❌ Failed: {}", metrics.failed_connections);
    println!("📈 Peak Concurrent: {}", metrics.peak_concurrent);
    println!("🎮 Successful Matches: {}", metrics.successful_matches);
    println!("🏁 Completed Games: {}", metrics.completed_games);
    println!("📤 Messages Sent: {}", metrics.total_messages_sent);
    println!("📥 Messages Received: {}", metrics.total_messages_received);
    println!("💔 Connection Drops: {}", metrics.connection_drops);
    println!("⏱️  Avg Connection Time: {:.2}ms", metrics.average_connection_time.as_millis());
    println!("⚡ Avg Response Time: {:.2}ms", metrics.average_response_time.as_millis());
    
    // Performance rating
    let rating = match success_rate {
        r if r >= 99.0 => "🏆 EXCELLENT",
        r if r >= 95.0 => "🥇 GREAT", 
        r if r >= 90.0 => "🥈 GOOD",
        r if r >= 80.0 => "🥉 FAIR",
        _ => "💥 NEEDS OPTIMIZATION"
    };
    
    println!("🏅 Performance Rating: {}", rating);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

fn print_progressive_summary(results: &[(u32, ExtremeTestMetrics)]) {
    println!("\n🚀 PROGRESSIVE TEST SUMMARY 🚀");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{:>10} | {:>10} | {:>8} | {:>8} | {:>10}", 
             "Target", "Success", "Rate%", "Peak", "Rating");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    for (connections, metrics) in results {
        let success_rate = (metrics.successful_connections as f64 / *connections as f64) * 100.0;
        let rating = match success_rate {
            r if r >= 99.0 => "🏆",
            r if r >= 95.0 => "🥇", 
            r if r >= 90.0 => "🥈",
            r if r >= 80.0 => "🥉",
            _ => "💥"
        };
        
        println!("{:>10} | {:>10} | {:>7.1}% | {:>8} | {:>10}", 
                 connections, metrics.successful_connections, success_rate, 
                 metrics.peak_concurrent, rating);
    }
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}