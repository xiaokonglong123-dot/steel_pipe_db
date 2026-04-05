mod db;

use axum::{
    extract::{Query, State, Json},
    http::StatusCode,
    response::{IntoResponse, Html},
    routing::{get, post, put, delete},
    Router,
};
use db::{Database, SteelPipe, InventoryRecord, DbError};
use serde::Deserialize;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use base64::Engine;
use std::io::Write;

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
}

#[derive(Deserialize)]
struct PipeQuery {
    page: Option<i64>,
    per_page: Option<i64>,
    search: Option<String>,
    material: Option<String>,
    status: Option<String>,
    min_diameter: Option<f64>,
    max_diameter: Option<f64>,
    min_length: Option<f64>,
    max_length: Option<f64>,
}

#[derive(Deserialize)]
struct RecordQuery {
    pipe_id: Option<String>,
    operation_type: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
}

#[derive(Deserialize)]
struct EntryRequest {
    pipe: SteelPipe,
    operator: String,
    remarks: Option<String>,
}

#[derive(Deserialize)]
struct ExitRequest {
    pipe_id: String,
    quantity: i32,
    operator: String,
    remarks: Option<String>,
}

#[derive(Deserialize)]
struct ImportRequest {
    csv_content: String,
    operator: String,
}

#[derive(Deserialize)]
struct ExcelImportRequest {
    excel_base64: String,
    operator: String,
}

#[derive(Deserialize)]
struct SaveRequest {
    path: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "pipes.db".to_string());
    let db = Arc::new(Database::new(&db_path).expect("Failed to initialize database"));
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
        .route("/api/pipes/entry", post(entry_pipe))
        .route("/api/pipes/exit", post(exit_pipe))
        .route("/api/statistics", get(get_statistics))
        .route("/api/material-stats", get(get_material_stats))
        .route("/api/low-stock", get(get_low_stock))
        .route("/api/records", get(list_records).post(create_record))
        .route("/api/logs", get(get_logs))
        .route("/api/export/inventory", get(export_inventory))
        .route("/api/export/inventory/excel", get(export_inventory_excel))
        .route("/api/export/records", get(export_records))
        .route("/api/export/records/excel", get(export_records_excel))
        .route("/api/import/csv", post(import_csv))
        .route("/api/import/excel", post(import_excel))
        .route("/api/save", post(save_database))
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

async fn list_pipes(
    State(state): State<AppState>,
    Query(q): Query<PipeQuery>,
) -> impl IntoResponse {
    let page = q.page.unwrap_or(1);
    let per_page = q.per_page.unwrap_or(20);
    match state.db.get_pipes(
        page, per_page,
        q.search.as_deref(),
        q.material.as_deref(),
        q.status.as_deref(),
        q.min_diameter, q.max_diameter,
        q.min_length, q.max_length,
    ).await {
        Ok((pipes, total)) => Json(serde_json::json!({
            "pipes": pipes,
            "total": total,
            "page": page,
            "per_page": per_page,
        })).into_response(),
        Err(e) => error_response(e),
    }
}

async fn create_pipe(
    State(state): State<AppState>,
    Json(pipe): Json<SteelPipe>,
) -> impl IntoResponse {
    match state.db.add_pipe(&pipe).await {
        Ok(()) => (StatusCode::CREATED, Json(serde_json::json!({"status": "created"}))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn get_pipe(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.db.get_pipe_by_id(&id).await {
        Ok(Some(pipe)) => Json(pipe).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "not found"}))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn update_pipe(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(mut pipe): Json<SteelPipe>,
) -> impl IntoResponse {
    pipe.pipe_id = id;
    match state.db.update_pipe(&pipe).await {
        Ok(()) => Json(serde_json::json!({"status": "updated"})).into_response(),
        Err(e) => error_response(e),
    }
}

async fn delete_pipe(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.db.delete_pipe(&id).await {
        Ok(()) => Json(serde_json::json!({"status": "deleted"})).into_response(),
        Err(e) => error_response(e),
    }
}

async fn entry_pipe(
    State(state): State<AppState>,
    Json(req): Json<EntryRequest>,
) -> impl IntoResponse {
    let qty = req.pipe.quantity;
    let pipe_id = req.pipe.pipe_id.clone();
    let operator = req.operator.clone();
    let remarks = req.remarks.clone().unwrap_or_default();

    match state.db.add_pipe(&req.pipe).await {
        Ok(()) => {
            let record = InventoryRecord {
                id: None,
                pipe_id: pipe_id.clone(),
                operation_type: "入库".to_string(),
                quantity: qty,
                operation_date: String::new(),
                operator: operator.clone(),
                remarks: Some(remarks.clone()),
            };
            if let Err(e) = state.db.add_inventory_record(&record).await {
                return error_response(e);
            }
            let _ = state.db.log_operation(
                "入库", "pipe", &pipe_id, "",
                &format!("{{\"qty\":{}}}", qty),
                &operator, &remarks,
            ).await;
            Json(serde_json::json!({"status": "created"})).into_response()
        }
        Err(e) => error_response(e),
    }
}

async fn exit_pipe(
    State(state): State<AppState>,
    Json(req): Json<ExitRequest>,
) -> impl IntoResponse {
    match state.db.get_pipe_by_id(&req.pipe_id).await {
        Ok(Some(pipe)) => {
            if pipe.quantity < req.quantity {
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": "库存不足",
                    "current": pipe.quantity
                }))).into_response();
            }
            let before = pipe.quantity;
            if let Err(e) = state.db.update_pipe_quantity(&req.pipe_id, -req.quantity).await {
                return error_response(e);
            }
            let record = InventoryRecord {
                id: None,
                pipe_id: req.pipe_id.clone(),
                operation_type: "出库".to_string(),
                quantity: req.quantity,
                operation_date: String::new(),
                operator: req.operator.clone(),
                remarks: req.remarks.clone(),
            };
            if let Err(e) = state.db.add_inventory_record(&record).await {
                return error_response(e);
            }
            let _ = state.db.log_operation(
                "出库", "pipe", &req.pipe_id,
                &format!("{{\"qty\":{}}}", before),
                &format!("{{\"qty\":{}}}", before - req.quantity),
                &req.operator,
                req.remarks.as_deref().unwrap_or(""),
            ).await;
            Json(serde_json::json!({"status": "success"})).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "未找到该钢管编号"}))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn get_statistics(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.get_statistics().await {
        Ok(stats) => Json(stats).into_response(),
        Err(e) => error_response(e),
    }
}

async fn get_material_stats(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.get_material_stats().await {
        Ok(stats) => Json(stats).into_response(),
        Err(e) => error_response(e),
    }
}

async fn get_low_stock(
    State(state): State<AppState>,
    Query(q): Query<std::collections::HashMap<String, i32>>,
) -> impl IntoResponse {
    let threshold = q.get("threshold").copied().unwrap_or(10);
    match state.db.get_low_stock(threshold).await {
        Ok(items) => Json(items).into_response(),
        Err(e) => error_response(e),
    }
}

async fn list_records(
    State(state): State<AppState>,
    Query(q): Query<RecordQuery>,
) -> impl IntoResponse {
    match state.db.get_inventory_records(
        q.pipe_id.as_deref(),
        q.operation_type.as_deref(),
        q.start_date.as_deref(),
        q.end_date.as_deref(),
    ).await {
        Ok(records) => Json(records).into_response(),
        Err(e) => error_response(e),
    }
}

async fn create_record(
    State(state): State<AppState>,
    Json(record): Json<InventoryRecord>,
) -> impl IntoResponse {
    match state.db.add_inventory_record(&record).await {
        Ok(()) => (StatusCode::CREATED, Json(serde_json::json!({"status": "created"}))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn get_logs(
    State(state): State<AppState>,
    Query(q): Query<std::collections::HashMap<String, usize>>,
) -> impl IntoResponse {
    let limit = q.get("limit").copied().unwrap_or(50);
    match state.db.get_operation_logs(limit).await {
        Ok(logs) => Json(logs).into_response(),
        Err(e) => error_response(e),
    }
}

async fn export_inventory(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.export_inventory_csv().await {
        Ok(csv) => (
            StatusCode::OK,
            [
                ("Content-Type", "text/csv; charset=utf-8"),
                ("Content-Disposition", "attachment; filename=\"inventory.csv\""),
            ],
            csv,
        ).into_response(),
        Err(e) => error_response(e),
    }
}

async fn export_records(
    State(state): State<AppState>,
    Query(q): Query<RecordQuery>,
) -> impl IntoResponse {
    match state.db.export_records_csv(
        q.pipe_id.as_deref(),
        q.operation_type.as_deref(),
        q.start_date.as_deref(),
        q.end_date.as_deref(),
    ).await {
        Ok(csv) => (
            StatusCode::OK,
            [
                ("Content-Type", "text/csv; charset=utf-8"),
                ("Content-Disposition", "attachment; filename=\"records.csv\""),
            ],
            csv,
        ).into_response(),
        Err(e) => error_response(e),
    }
}

async fn import_csv(
    State(state): State<AppState>,
    Json(req): Json<ImportRequest>,
) -> impl IntoResponse {
    match state.db.import_pipes_from_csv(&req.csv_content, &req.operator).await {
        Ok((success, fail)) => Json(serde_json::json!({
            "success": success,
            "fail": fail,
        })).into_response(),
        Err(e) => error_response(e),
    }
}

async fn save_database(
    State(state): State<AppState>,
    Json(req): Json<SaveRequest>,
) -> impl IntoResponse {
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "pipes.db".to_string());
    let path = req.path.unwrap_or_else(|| format!("pipes_backup_{}.db", chrono::Local::now().format("%Y%m%d_%H%M%S")));
    match tokio::fs::copy(&db_path, &path).await {
        Ok(_) => Json(serde_json::json!({"status": "saved", "path": path})).into_response(),
        Err(e) => error_response(DbError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))),
    }
}

async fn export_inventory_excel(State(state): State<AppState>) -> impl IntoResponse {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join(format!("inventory_{}.xlsx", chrono::Local::now().format("%Y%m%d_%H%M%S")));
    let path_str = path.to_string_lossy().to_string();
    match state.db.export_inventory_to_excel(&path_str).await {
        Ok(()) => {
            match tokio::fs::read(&path).await {
                Ok(data) => {
                    let _ = tokio::fs::remove_file(&path).await;
                    (
                        StatusCode::OK,
                        [
                            ("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
                            ("Content-Disposition", "attachment; filename=\"inventory.xlsx\""),
                        ],
                        data,
                    ).into_response()
                }
                Err(e) => error_response(DbError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
            }
        }
        Err(e) => error_response(e),
    }
}

async fn export_records_excel(
    State(state): State<AppState>,
    Query(q): Query<RecordQuery>,
) -> impl IntoResponse {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join(format!("records_{}.xlsx", chrono::Local::now().format("%Y%m%d_%H%M%S")));
    let path_str = path.to_string_lossy().to_string();
    match state.db.export_records_to_excel(&path_str, q.pipe_id.as_deref(), q.operation_type.as_deref(), q.start_date.as_deref(), q.end_date.as_deref()).await {
        Ok(()) => {
            match tokio::fs::read(&path).await {
                Ok(data) => {
                    let _ = tokio::fs::remove_file(&path).await;
                    (
                        StatusCode::OK,
                        [
                            ("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
                            ("Content-Disposition", "attachment; filename=\"records.xlsx\""),
                        ],
                        data,
                    ).into_response()
                }
                Err(e) => error_response(DbError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
            }
        }
        Err(e) => error_response(e),
    }
}

async fn import_excel(
    State(state): State<AppState>,
    Json(req): Json<ExcelImportRequest>,
) -> impl IntoResponse {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&req.excel_base64)
        .map_err(|e| DbError::Validation(e.to_string()))?;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("import_{}.xlsx", chrono::Local::now().format("%Y%m%d_%H%M%S")));
    let temp_path_str = temp_path.to_string_lossy().to_string();

    if let Ok(mut file) = std::fs::File::create(&temp_path) {
        if file.write_all(&decoded).is_ok() {
            match state.db.import_pipes_from_excel(&temp_path_str, &req.operator).await {
                Ok((success, fail)) => {
                    return Json(serde_json::json!({
                        "success": success,
                        "fail": fail,
                    })).into_response();
                }
                Err(e) => return error_response(e),
            }
        }
    }
    error_response(DbError::Validation("Failed to write temp file".to_string()))
}

fn error_response(e: DbError) -> axum::response::Response {
    let status = match &e {
        DbError::NotFound(_) => StatusCode::NOT_FOUND,
        DbError::Validation(_) => StatusCode::BAD_REQUEST,
        DbError::InsufficientStock { .. } => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, Json(serde_json::json!({"error": e.to_string()}))).into_response()
}
