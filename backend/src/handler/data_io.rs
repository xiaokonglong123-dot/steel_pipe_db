use std::sync::Arc;

use axum::{
    extract::{Multipart, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use serde_json::json;

use crate::error::{AppError, AppResult};
use crate::middleware::AuthUser;
use crate::service::data_io_service::{ExportFilter, ImportConfig};
use crate::AppState;

// ── Import handlers ─────────────────────────────────────────────────────────

pub async fn import_seamless_pipes(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // Permission check: admin, warehouse
    if auth.role != "admin" && auth.role != "warehouse" {
        return Err(AppError::Forbidden(
            "Only admin and warehouse roles can import pipes".into(),
        ));
    }

    let (data, content_type, config) = extract_import_params(&mut multipart).await?;
    let result = state.data_io_service.import_seamless_pipes(data, &content_type, &config).await?;

    Ok(Json(json!({
        "success": true,
        "data": result,
        "request_id": ""
    })))
}

pub async fn import_screen_pipes(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" && auth.role != "warehouse" {
        return Err(AppError::Forbidden(
            "Only admin and warehouse roles can import pipes".into(),
        ));
    }

    let (data, content_type, config) = extract_import_params(&mut multipart).await?;
    let result = state.data_io_service.import_screen_pipes(data, &content_type, &config).await?;

    Ok(Json(json!({
        "success": true,
        "data": result,
        "request_id": ""
    })))
}

// ── Template download handlers ───────────────────────────────────────────────

pub async fn download_seamless_template(
    State(state): State<Arc<AppState>>,
) -> AppResult<Response> {
    let bytes = state.data_io_service.generate_seamless_template()?;
    Ok(file_response(bytes, "seamless_pipe_import_template.xlsx", "xlsx"))
}

pub async fn download_screen_template(
    State(state): State<Arc<AppState>>,
) -> AppResult<Response> {
    let bytes = state.data_io_service.generate_screen_template()?;
    Ok(file_response(bytes, "screen_pipe_import_template.xlsx", "xlsx"))
}

// ── Export handlers ─────────────────────────────────────────────────────────

pub async fn export_inventory(
    State(state): State<Arc<AppState>>,
    Json(filter): Json<ExportFilter>,
) -> AppResult<Response> {
    let bytes = state.data_io_service.export_inventory(&filter).await?;
    let ext = if filter.fmt() == "csv" { "csv" } else { "xlsx" };
    Ok(file_response(bytes, &format!("inventory_export.{}", ext), ext))
}

pub async fn export_inbound(
    State(state): State<Arc<AppState>>,
    Json(filter): Json<ExportFilter>,
) -> AppResult<Response> {
    let bytes = state.data_io_service.export_inbound(&filter).await?;
    let ext = if filter.fmt() == "csv" { "csv" } else { "xlsx" };
    Ok(file_response(bytes, &format!("inbound_export.{}", ext), ext))
}

pub async fn export_outbound(
    State(state): State<Arc<AppState>>,
    Json(filter): Json<ExportFilter>,
) -> AppResult<Response> {
    let bytes = state.data_io_service.export_outbound(&filter).await?;
    let ext = if filter.fmt() == "csv" { "csv" } else { "xlsx" };
    Ok(file_response(bytes, &format!("outbound_export.{}", ext), ext))
}

pub async fn export_pipes(
    State(state): State<Arc<AppState>>,
    Json(filter): Json<ExportFilter>,
) -> AppResult<Response> {
    let bytes = state.data_io_service.export_pipes(&filter).await?;
    let ext = if filter.fmt() == "csv" { "csv" } else { "xlsx" };
    Ok(file_response(bytes, &format!("pipes_export.{}", ext), ext))
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Extract file data, content-type, and optional import config from multipart form.
async fn extract_import_params(
    multipart: &mut Multipart,
) -> AppResult<(Vec<u8>, String, ImportConfig)> {
    let mut data = None;
    let mut content_type = String::new();
    let mut config = ImportConfig::default();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                content_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::BadRequest(format!("File read error: {}", e)))?
                        .to_vec(),
                );
            }
            "on_duplicate" => {
                let val = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Field error: {}", e)))?;
                if val.trim().eq_ignore_ascii_case("overwrite") {
                    config.on_duplicate = crate::service::data_io_service::DuplicateStrategy::Overwrite;
                }
            }
            _ => {}
        }
    }

    let file_data = data.ok_or_else(|| AppError::BadRequest("Missing 'file' field in multipart form".into()))?;

    if file_data.is_empty() {
        return Err(AppError::BadRequest("Uploaded file is empty".into()));
    }

    Ok((file_data, content_type, config))
}

/// Build a file download response with appropriate headers.
fn file_response(bytes: Vec<u8>, filename: &str, ext: &str) -> Response {
    let mime = match ext {
        "csv" => "text/csv; charset=utf-8",
        _ => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(axum::body::Body::from(bytes))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(axum::body::Body::from("Failed to build response"))
                .unwrap()
        })
}
