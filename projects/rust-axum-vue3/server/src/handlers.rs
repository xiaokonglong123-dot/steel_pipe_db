use axum::{
    extract::{Query, State, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use crate::db::Database;
use crate::models::*;
use crate::error::{AppError, Result};
use std::sync::Arc;
use base64::Engine;
use std::io::Write;
use chrono::Datelike;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
}

pub async fn list_pipes(
    State(state): State<AppState>,
    Query(q): Query<PipeQuery>,
) -> Result<impl IntoResponse> {
    let page = q.page.unwrap_or(1);
    let per_page = q.per_page.unwrap_or(20);
    let (pipes, total) = state.db.get_pipes(
        page, per_page,
        q.search,
        q.material,
        q.status,
        q.min_diameter, q.max_diameter,
        q.min_length, q.max_length,
    ).await?;
    
    Ok(Json(serde_json::json!({
        "pipes": pipes,
        "total": total,
        "page": page,
        "per_page": per_page,
    })))
}

pub async fn create_pipe(
    State(state): State<AppState>,
    Json(pipe): Json<SteelPipe>,
) -> Result<impl IntoResponse> {
    pipe.validate()?;
    state.db.add_pipe(&pipe).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"status": "created"}))))
}

pub async fn get_pipe(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    match state.db.get_pipe_by_id(id).await? {
        Some(pipe) => Ok(Json(pipe).into_response()),
        None => Err(AppError::NotFound("未找到该钢管".to_string())),
    }
}

pub async fn update_pipe(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut pipe): Json<SteelPipe>,
) -> Result<impl IntoResponse> {
    pipe.pipe_id = id;
    state.db.update_pipe(pipe).await?;
    Ok(Json(serde_json::json!({"status": "updated"})))
}

pub async fn delete_pipe(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    state.db.delete_pipe(id).await?;
    Ok(Json(serde_json::json!({"status": "deleted"})))
}

pub async fn batch_delete_pipes(
    State(state): State<AppState>,
    Json(req): Json<BatchDeleteRequest>,
) -> Result<impl IntoResponse> {
    let count = req.pipe_ids.len();
    state.db.batch_delete_pipes(req.pipe_ids.clone()).await?;
    
    let _ = state.db.log_operation(
        "批量删除", "pipe", "批量",
        &format!("{{\"count\":{}}}", count),
        "",
        &req.operator,
        &req.pipe_ids.join(","),
    ).await;
    
    Ok(Json(serde_json::json!({"status": "deleted", "count": count})))
}

pub async fn entry_pipe(
    State(state): State<AppState>,
    Json(req): Json<EntryRequest>,
) -> Result<impl IntoResponse> {
    req.pipe.validate()?;
    
    let pipe_id = req.pipe.pipe_id.clone();
    let qty = req.pipe.quantity;
    let operator = req.operator.clone();
    let remarks = req.remarks.clone();

    state.db.process_entry(req.pipe, req.operator, req.remarks.clone()).await?;
    
    let _ = state.db.log_operation(
        "入库", "pipe", &pipe_id, "",
        &format!("{{\"qty\":{}}}", qty),
        &operator, &remarks.unwrap_or_default(),
    ).await;
    
    Ok(Json(serde_json::json!({"status": "created"})))
}

pub async fn exit_pipe(
    State(state): State<AppState>,
    Json(req): Json<ExitRequest>,
) -> Result<impl IntoResponse> {
    if req.pipe_id.trim().is_empty() {
        return Err(AppError::Validation("钢管编号不能为空".to_string()));
    }
    if req.quantity <= 0 {
        return Err(AppError::Validation("数量必须大于0".to_string()));
    }
    
    let (before, after) = state.db.process_exit(req.pipe_id.clone(), req.quantity, req.operator.clone(), req.remarks.clone()).await?;
    
    let _ = state.db.log_operation(
        "出库", "pipe", &req.pipe_id,
        &format!("{{\"qty\":{}}}", before),
        &format!("{{\"qty\":{}}}", after),
        &req.operator,
        req.remarks.as_deref().unwrap_or(""),
    ).await;
    
    Ok(Json(serde_json::json!({"status": "success"})))
}

pub async fn get_statistics(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let stats = state.db.get_statistics().await?;
    Ok(Json(stats))
}

pub async fn get_material_stats(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let stats = state.db.get_material_stats().await?;
    Ok(Json(stats))
}

pub async fn get_low_stock(
    State(state): State<AppState>,
    Query(q): Query<std::collections::HashMap<String, i32>>,
) -> Result<impl IntoResponse> {
    let threshold = q.get("threshold").copied().unwrap_or(10);
    let items = state.db.get_low_stock(threshold).await?;
    Ok(Json(items))
}

pub async fn get_dicts(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let dicts = state.db.get_data_dictionaries().await?;
    Ok(Json(dicts))
}

pub async fn get_trends(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let trends = state.db.get_inventory_trends().await?;
    Ok(Json(trends))
}

pub async fn list_records(
    State(state): State<AppState>,
    Query(q): Query<RecordQuery>,
) -> Result<impl IntoResponse> {
    let records = state.db.get_inventory_records(
        q.pipe_id,
        q.operation_type,
        q.start_date,
        q.end_date,
    ).await?;
    Ok(Json(records))
}

pub async fn create_record(
    State(state): State<AppState>,
    Json(record): Json<InventoryRecord>,
) -> Result<impl IntoResponse> {
    state.db.add_inventory_record(record).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"status": "created"}))))
}

pub async fn list_productions(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let productions = state.db.get_productions().await?;
    Ok(Json(productions))
}

pub async fn create_production(
    State(state): State<AppState>,
    Json(req): Json<ProductionRequest>,
) -> Result<impl IntoResponse> {
    let production = Production {
        id: None,
        furnace_number: req.furnace_number,
        heat_treatment_batch: req.heat_treatment_batch,
        material_batch: req.material_batch,
        production_count: req.production_count,
        sample: req.sample,
        supplier: req.supplier,
        operator: req.operator,
        production_date: String::new(),
        remarks: req.remarks,
    };
    let id = state.db.add_production(production).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id, "status": "created"}))))
}

pub async fn get_logs(
    State(state): State<AppState>,
    Query(q): Query<std::collections::HashMap<String, usize>>,
) -> Result<impl IntoResponse> {
    let limit = q.get("limit").copied().unwrap_or(50);
    let logs = state.db.get_operation_logs(limit).await?;
    Ok(Json(logs))
}

pub async fn export_inventory(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let csv = state.db.export_inventory_csv().await?;
    Ok((
        StatusCode::OK,
        [
            ("Content-Type", "text/csv; charset=utf-8"),
            ("Content-Disposition", "attachment; filename=\"inventory.csv\""),
        ],
        csv,
    ))
}

pub async fn export_records(
    State(state): State<AppState>,
    Query(q): Query<RecordQuery>,
) -> Result<impl IntoResponse> {
    let csv = state.db.export_records_csv(
        q.pipe_id,
        q.operation_type,
        q.start_date,
        q.end_date,
    ).await?;
    Ok((
        StatusCode::OK,
        [
            ("Content-Type", "text/csv; charset=utf-8"),
            ("Content-Disposition", "attachment; filename=\"records.csv\""),
        ],
        csv,
    ))
}

pub async fn batch_export_pipes(
    State(state): State<AppState>,
    Query(q): Query<PipeQuery>,
) -> Result<impl IntoResponse> {
    let csv = state.db.export_pipes_by_filter(
        q.search,
        q.material,
        q.status,
        q.min_diameter, q.max_diameter,
        q.min_length, q.max_length,
    ).await?;
    Ok((
        StatusCode::OK,
        [
            ("Content-Type", "text/csv; charset=utf-8"),
            ("Content-Disposition", "attachment; filename=\"pipes_export.csv\""),
        ],
        csv,
    ))
}

pub async fn import_csv(
    State(state): State<AppState>,
    Json(req): Json<ImportRequest>,
) -> Result<impl IntoResponse> {
    let (success, fail) = state.db.import_pipes_from_csv(req.csv_content, req.operator).await?;
    Ok(Json(serde_json::json!({
        "success": success,
        "fail": fail,
    })))
}

pub async fn save_database(
    State(_state): State<AppState>,
    Json(req): Json<SaveRequest>,
) -> Result<impl IntoResponse> {
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "pipes.db".to_string());
    let path = req.path.unwrap_or_else(|| format!("pipes_backup_{}.db", chrono::Local::now().format("%Y%m%d_%H%M%S")));
    tokio::fs::copy(&db_path, &path).await?;
    Ok(Json(serde_json::json!({"status": "saved", "path": path})))
}

pub async fn restore_database(
    State(_state): State<AppState>,
    Json(req): Json<RestoreRequest>,
) -> Result<impl IntoResponse> {
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "pipes.db".to_string());
    
    if !tokio::fs::try_exists(&req.backup_path).await.unwrap_or(false) {
        return Err(AppError::Validation("备份文件不存在".to_string()));
    }
    
    tokio::fs::copy(&req.backup_path, &db_path).await?;
    Ok(Json(serde_json::json!({"status": "restored", "path": db_path})))
}

pub async fn daily_report(
    State(state): State<AppState>,
    Query(_q): Query<ReportRequest>,
) -> Result<impl IntoResponse> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let report = state.db.get_daily_report(today.clone(), today).await?;
    Ok(Json(report))
}

pub async fn monthly_report(
    State(state): State<AppState>,
    Query(_q): Query<ReportRequest>,
) -> Result<impl IntoResponse> {
    let now = chrono::Local::now();
    let start = now.with_day(1).unwrap().format("%Y-%m-%d").to_string();
    let end = now.format("%Y-%m-%d").to_string();
    let report = state.db.get_monthly_report(start, end).await?;
    Ok(Json(report))
}

pub async fn export_inventory_excel(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join(format!("inventory_{}.xlsx", chrono::Local::now().format("%Y%m%d_%H%M%S")));
    let path_str = path.to_string_lossy().to_string();
    
    state.db.export_inventory_to_excel(path_str.clone()).await?;
    
    let data = tokio::fs::read(&path).await?;
    let _ = tokio::fs::remove_file(&path).await;
    
    Ok((
        StatusCode::OK,
        [
            ("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
            ("Content-Disposition", "attachment; filename=\"inventory.xlsx\""),
        ],
        data,
    ))
}

pub async fn export_records_excel(
    State(state): State<AppState>,
    Query(q): Query<RecordQuery>,
) -> Result<impl IntoResponse> {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join(format!("records_{}.xlsx", chrono::Local::now().format("%Y%m%d_%H%M%S")));
    let path_str = path.to_string_lossy().to_string();
    
    state.db.export_records_to_excel(
        path_str.clone(),
        q.pipe_id,
        q.operation_type,
        q.start_date,
        q.end_date,
    ).await?;
    
    let data = tokio::fs::read(&path).await?;
    let _ = tokio::fs::remove_file(&path).await;
    
    Ok((
        StatusCode::OK,
        [
            ("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
            ("Content-Disposition", "attachment; filename=\"records.xlsx\""),
        ],
        data,
    ))
}

pub async fn import_excel(
    State(state): State<AppState>,
    Json(req): Json<ExcelImportRequest>,
) -> Result<impl IntoResponse> {
    let decoded = base64::engine::general_purpose::STANDARD.decode(&req.excel_base64)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("import_{}.xlsx", chrono::Local::now().format("%Y%m%d_%H%M%S")));
    let temp_path_str = temp_path.to_string_lossy().to_string();

    let mut file = std::fs::File::create(&temp_path)?;
    file.write_all(&decoded)?;
    
    let (success, fail) = state.db.import_pipes_from_excel(temp_path_str, req.operator).await?;
    
    Ok(Json(serde_json::json!({
        "success": success,
        "fail": fail,
    })))
}
