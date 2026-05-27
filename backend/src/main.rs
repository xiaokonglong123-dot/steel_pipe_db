#![allow(dead_code)]

use std::net::SocketAddr;

use sqlx::sqlite::SqlitePoolOptions;
use tracing_subscriber::EnvFilter;

mod config;
mod domain;
mod dto;
mod error;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod response;
mod router;
mod services;

#[tokio::main]
async fn main() {
    // Tracing must be initialized before any logging — panic hooks capture early crashes
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    // Load .env before config — env vars must be present before from_env() reads them
    dotenvy::dotenv().ok();

    // Read all env-based config upfront — panics early if critical vars are missing
    let cfg = config::Config::from_env();

    // Pool must be created before routes — all handlers pull connections from this pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&cfg.database_url)
        .await
        .expect("Failed to connect to database");

    // Migrations must run before the server starts — stale schema causes runtime errors
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("Database migrations completed");

    // Assemble the full router tree — all middleware and route groups merge here
    let app = router::create_app(pool, cfg.jwt_secret.clone());

    // Bind and serve — axum::serve is the outermost layer that drives the async event loop
    let addr: SocketAddr = cfg
        .server_addr()
        .parse()
        .expect("Invalid server address");

    tracing::info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
