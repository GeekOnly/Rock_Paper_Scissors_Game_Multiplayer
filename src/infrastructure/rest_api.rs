use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;
use warp::Filter;

use crate::application::GameManager;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub active_rooms: usize,
    pub waiting_players: usize,
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub total_rooms: usize,
    pub active_games: usize,
    pub waiting_players: usize,
}

pub fn create_routes(
    game_manager: Arc<GameManager>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let health = warp::path("health")
        .and(warp::get())
        .and(with_game_manager(game_manager.clone()))
        .and_then(health_handler);

    let stats = warp::path("stats")
        .and(warp::get())
        .and(with_game_manager(game_manager))
        .and_then(stats_handler);

    health.or(stats)
}

fn with_game_manager(
    game_manager: Arc<GameManager>,
) -> impl Filter<Extract = (Arc<GameManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || game_manager.clone())
}

async fn health_handler(
    game_manager: Arc<GameManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let (total_rooms, _, waiting_players) = game_manager.get_stats().await;

    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        active_rooms: total_rooms,
        waiting_players,
    };

    Ok(warp::reply::json(&response))
}

async fn stats_handler(game_manager: Arc<GameManager>) -> Result<impl warp::Reply, warp::Rejection> {
    let (total_rooms, active_games, waiting_players) = game_manager.get_stats().await;

    let response = StatsResponse {
        total_rooms,
        active_games,
        waiting_players,
    };

    Ok(warp::reply::json(&response))
}