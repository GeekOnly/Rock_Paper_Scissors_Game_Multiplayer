use anyhow::Result;
use clap::{Arg, Command};
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber;

use rps_server::tests::{test_concurrent_connections, test_connection_limits, LoadTestConfig, LoadTestRunner};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let matches = Command::new("RPS Load Test")
        .version("1.0")
        .about("Load testing tool for RPS Game Server")
        .arg(
            Arg::new("connections")
                .short('c')
                .long("connections")
                .value_name("NUMBER")
                .help("Number of concurrent connections")
                .default_value("100"),
        )
        .arg(
            Arg::new("duration")
                .short('d')
                .long("duration")
                .value_name("SECONDS")
                .help("Test duration in seconds")
                .default_value("30"),
        )
        .arg(
            Arg::new("server")
                .short('s')
                .long("server")
                .value_name("URL")
                .help("Server WebSocket URL")
                .default_value("ws://127.0.0.1:8080"),
        )
        .arg(
            Arg::new("test-type")
                .short('t')
                .long("test-type")
                .value_name("TYPE")
                .help("Type of test to run")
                .value_parser(["concurrent", "limits", "sustained", "custom"])
                .default_value("concurrent"),
        )
        .get_matches();

    let connections: usize = matches.get_one::<String>("connections").unwrap().parse()?;
    let duration: u64 = matches.get_one::<String>("duration").unwrap().parse()?;
    let server_url = matches.get_one::<String>("server").unwrap().clone();
    let test_type = matches.get_one::<String>("test-type").unwrap();

    info!("🚀 Starting RPS Load Test");
    info!("Server: {}", server_url);
    info!("Test Type: {}", test_type);

    match test_type.as_str() {
        "concurrent" => {
            info!("Testing {} concurrent connections", connections);
            let metrics = test_concurrent_connections(connections).await?;
            print_metrics(&metrics);
        }
        "limits" => {
            info!("Testing connection limits");
            let results = test_connection_limits().await?;
            print_limit_results(&results);
        }
        "sustained" => {
            info!("Running sustained load test for {} seconds", duration);
            let config = LoadTestConfig {
                concurrent_connections: connections,
                test_duration: Duration::from_secs(duration),
                server_url,
                ..Default::default()
            };
            let runner = LoadTestRunner::new(config);
            let metrics = runner.run_load_test().await?;
            print_metrics(&metrics);
        }
        "custom" => {
            info!("Running custom load test");
            let config = LoadTestConfig {
                concurrent_connections: connections,
                test_duration: Duration::from_secs(duration),
                server_url,
                connection_timeout: Duration::from_secs(10),
                message_timeout: Duration::from_secs(15),
            };
            let runner = LoadTestRunner::new(config);
            let metrics = runner.run_load_test().await?;
            print_metrics(&metrics);
        }
        _ => {
            eprintln!("Unknown test type: {}", test_type);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_metrics(metrics: &rps_server::tests::LoadTestMetrics) {
    println!("\n📊 Load Test Results:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔗 Connections:");
    println!("  ✅ Successful: {}", metrics.successful_connections);
    println!("  ❌ Failed: {}", metrics.failed_connections);
    
    let total_connections = metrics.successful_connections + metrics.failed_connections;
    if total_connections > 0 {
        let success_rate = (metrics.successful_connections as f64 / total_connections as f64) * 100.0;
        println!("  📈 Success Rate: {:.2}%", success_rate);
    }

    println!("\n🎮 Matchmaking:");
    println!("  ✅ Successful Matches: {}", metrics.successful_matches);
    println!("  ❌ Failed Matches: {}", metrics.failed_matches);
    println!("  🏁 Completed Games: {}", metrics.completed_games);

    println!("\n📨 Messages:");
    println!("  📤 Sent: {}", metrics.total_messages_sent);
    println!("  📥 Received: {}", metrics.total_messages_received);

    println!("\n⏱️  Performance:");
    println!("  🔗 Avg Connection Time: {:?}", metrics.average_connection_time);
    println!("  🎯 Avg Match Time: {:?}", metrics.average_match_time);

    if !metrics.errors.is_empty() {
        println!("\n❌ Errors:");
        for error in &metrics.errors {
            println!("  • {}", error);
        }
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

fn print_limit_results(results: &[(usize, rps_server::tests::LoadTestMetrics)]) {
    println!("\n📊 Connection Limit Test Results:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{:>12} | {:>10} | {:>10} | {:>12}", "Connections", "Successful", "Failed", "Success Rate");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut max_successful = 0;
    for (connections, metrics) in results {
        let success_rate = if *connections > 0 {
            (metrics.successful_connections as f64 / *connections as f64) * 100.0
        } else {
            0.0
        };
        
        println!("{:>12} | {:>10} | {:>10} | {:>11.1}%", 
                 connections, 
                 metrics.successful_connections, 
                 metrics.failed_connections, 
                 success_rate);
        
        if success_rate >= 90.0 {
            max_successful = *connections;
        }
    }
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🏆 Maximum successful concurrent connections: {}", max_successful);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}