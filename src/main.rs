use axum::middleware;
use axum::routing::{get, post};
use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse, Router};
use dashmap_cache::DashmapCache;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time;
use stores::database::Repository;
use tower_sessions::Expiry;
use tower_sessions::MemoryStore;
use tower_sessions::SessionManagerLayer;

use crate::stores::events::Events;
// pub mod config
pub mod abilities;
pub mod charclasses;
pub mod map;
pub mod rest;
pub mod schemas;
pub mod services;
pub mod stores;

#[derive(Clone)]
pub struct Repositories {
    pub cache: Arc<DashmapCache>,
    pub events: Events,
    pub db: Repository,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 12)]
async fn main() {
    tracing_subscriber::fmt().json().init();
    /*
    let config = Config::from_env();
    */
    let repos = Repositories {
        db: Repository::new().await,
        cache: Arc::new(DashmapCache::new()),
        events: Events::new(),
    };

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_expiry(Expiry::OnSessionEnd);

    let app = Router::new()
        .route("/login", post(rest::login))
        .route("/create_user", post(rest::create_user))
        .route("/logout", post(rest::logout))
        .route("/game", get(rest::get_active_game))
        .route("/game", post(rest::new_game))
        .route("/scenario", get(rest::get_scenarios))
        .route("/scenario/:scenario_id", get(rest::get_scenario_players))
        .route(
            "/game/:game_id/scenario_players",
            get(rest::get_available_scenario_players),
        )
        .route("/game/:game_id/deploy", post(rest::deploy_entities))
        .route(
            "/game/:game_id/ability/:ability_name",
            post(rest::use_ability),
        )
        .route(
            "/game/:game_id/entity/transfer",
            post(rest::transfer_entity),
        )
        .route("/game/:game_id/ws", get(rest::ws_handler))
        .layer(session_layer)
        .layer(middleware::from_fn(log_access))
        .with_state(repos);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8061));
    tracing::debug!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn log_access(req: Request, next: Next) -> Result<impl IntoResponse, (StatusCode, String)> {
    let t0 = time::Instant::now();
    let uri = req.uri().to_owned().to_string();

    let res = next.run(req).await;

    let t = (time::Instant::now() - t0).as_millis();
    let (head, body) = res.into_parts();
    tracing::info!(
        uri = uri,
        time_ms = t,
        status = head.status.as_u16().clone()
    );
    Ok((head, body).into_response())
}
