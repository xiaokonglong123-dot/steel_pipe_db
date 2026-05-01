mod db;
mod models;
mod handlers;
mod error;

use axum::{
    response::Html,
    routing::{get, post},
    Router,
};
use handlers::*;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "pipes.db".to_string());
    let db = Arc::new(db::Database::new(&db_path).expect("Failed to initialize database"));
    let state = AppState { db };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let frontend = ServeDir::new("../web/dist").not_found_service(
        axum::routing::get(|| async {
            let content = tokio::fs::read_to_string("../web/dist/index.html").await.unwrap_or_default();
            Html(content)
        })
    );

    let app = Router::new()
        .route("/api/pipes", get(list_pipes).post(create_pipe))
        .route("/api/pipes/:id", get(get_pipe).put(update_pipe).delete(delete_pipe))
        .route("/api/pipes/batch-delete", post(batch_delete_pipes))
        .route("/api/pipes/entry", post(entry_pipe))
        .route("/api/pipes/exit", post(exit_pipe))
        .route("/api/pipes/batch-export", get(batch_export_pipes))
        .route("/api/statistics", get(get_statistics))
        .route("/api/material-stats", get(get_material_stats))
        .route("/api/low-stock", get(get_low_stock))
        .route("/api/trends", get(get_trends))
        .route("/api/dicts", get(get_dicts))
        .route("/api/productions", get(list_productions).post(create_production))
        .route("/api/records", get(list_records).post(create_record))
        .route("/api/logs", get(get_logs))
        .route("/api/export/inventory", get(export_inventory))
        .route("/api/export/inventory/excel", get(export_inventory_excel))
        .route("/api/export/records", get(export_records))
        .route("/api/export/records/excel", get(export_records_excel))
        .route("/api/import/csv", post(import_csv))
        .route("/api/import/excel", post(import_excel))
        .route("/api/save", post(save_database))
        .route("/api/restore", post(restore_database))
        .route("/api/report/daily", get(daily_report))
        .route("/api/report/monthly", get(monthly_report))
        .layer(cors)
        .nest_service("/", frontend)
        .with_state(state);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);
    let addr = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", addr, e));

    tracing::info!("Server running on http://{}", addr);
    axum::serve(listener, app).await.expect("Server failed");
}
