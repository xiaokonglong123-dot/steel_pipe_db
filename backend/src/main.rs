#![allow(dead_code)]

use std::net::SocketAddr;

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

use crate::dto::auth_dto::CreateUserRequest;
use crate::repositories::user_repo::UserRepo;
use crate::services::auth_service::AuthService;

mod auth;
mod cache;
mod cache_invalidator;
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
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Load .env before config — env vars must be present before from_env() reads them
    dotenvy::dotenv().ok();

    // Read all env-based config upfront — panics early if critical vars are missing
    let cfg = config::Config::from_env();

    // Pool must be created before routes — all handlers pull connections from this pool
    let pool = PgPoolOptions::new()
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

    // Bootstrap the initial admin user if no users exist yet.
    // This replaces the old migration-seeded hardcoded admin credential.
    bootstrap_admin(&pool, &cfg.admin_username, &cfg.admin_password).await;

    // Create the cache manager — holds typed caches for grades, locations, dashboard
    let cache_manager = crate::cache::CacheManager::new();
    tracing::info!("Cache manager initialized (grades=5min, locations=2min, dashboard=30s)");

    // Create the cache invalidator for event-driven cache invalidation
    let cache_invalidator = crate::cache_invalidator::CacheInvalidator::new(cache_manager.clone());
    tracing::info!("Cache invalidator initialized (event-driven)");

    // Initialize default invalidation rules
    let registry = crate::cache_invalidator::CacheInvalidationRegistry::new();
    crate::cache_invalidator::init_default_invalidation_rules(&registry);

    // Assemble the full router tree — all middleware and route groups merge here
    let cors_origins = cfg.parse_cors_origins();
    tracing::info!("CORS origins: {:?}", cors_origins);
    let app = router::create_app(pool, cfg.jwt_secret.clone(), cors_origins, cache_manager, cache_invalidator);

    // Bind and serve — axum::serve is the outermost layer that drives the async event loop
    let addr: SocketAddr = cfg.server_addr().parse().expect("Invalid server address");

    tracing::info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server failed");
}

/// Creates the initial admin user when the database is empty.
async fn bootstrap_admin(pool: &sqlx::PgPool, admin_username: &str, admin_password: &str) {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    if count > 0 {
        tracing::info!("Admin user exists — skipping bootstrap");
        return;
    }

    let password_hash = match AuthService::hash_password(admin_password) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("Failed to hash admin password: {}", e);
            return;
        }
    };

    let dto = CreateUserRequest {
        username: admin_username.to_string(),
        password: admin_password.to_string(),
        display_name: "Administrator".to_string(),
        role: "admin".to_string(),
        email: None,
        phone: None,
    };

    match UserRepo::create(pool, &dto, &password_hash).await {
        Ok(user) => tracing::info!(
            "Bootstrapped admin user '{}' (id={})",
            user.username,
            user.id
        ),
        Err(e) => tracing::error!("Failed to bootstrap admin user: {}", e),
    }
}
