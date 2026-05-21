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
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    // Load .env file
    dotenvy::dotenv().ok();

    // Load config
    let cfg = config::Config::from_env();

    // Initialize database pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&cfg.database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("Database migrations completed");

    // Build application
    let app = router::create_app(pool, cfg.jwt_secret.clone());

    // Start server
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
