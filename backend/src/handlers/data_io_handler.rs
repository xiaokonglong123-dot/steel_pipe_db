use axum::{
    extract::{Extension, Multipart, Path, Query},
    http::header,
    response::IntoResponse,
    Json,
};
use sqlx::SqlitePool;
use validator::Validate;

use crate::dto::data_io_dto::*;
use crate::error::AppError;
use crate::repositories::operation_log_repo::OperationLog;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::data_io_service::DataIOService;

/// POST `/api/v1/data/{entity_type}/import` — Import data (Excel/CSV)
///
/// Accepts a multipart file upload for a given entity type.
/// Logs the import operation. Returns import stats (imported/failed counts).
pub async fn import_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(entity_type): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<ImportResult>>, AppError> {
    let entity_type = entity_type.to_lowercase();

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::ImportError(format!("Read multipart: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            file_name = Some(field.file_name().unwrap_or("import").to_string());
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::ImportError(format!("Read file: {}", e)))?
                    .to_vec(),
            );
        }
    }

    let data = file_data.ok_or_else(|| AppError::ImportError("No file field found in upload".into()))?;
    let fname = file_name.unwrap_or_else(|| "import.xlsx".to_string());

    let result = DataIOService::import_entity(&pool, &entity_type, &data, &fname).await?;

    DataIOService::log_operation(
        &pool,
        None,
        None,
        "import",
        &entity_type,
        None,
        Some(serde_json::json!({
            "imported": result.imported_count,
            "failed": result.failed_count,
            "file": fname,
        }).to_string()),
        None,
    )
    .await?;

    Ok(ApiResponse::ok(result))
}

/// GET `/api/v1/data/{entity_type}/export` — Export data (Excel/CSV)
///
/// Exports all records for a given entity type in the requested format.
/// Logs the export operation. Returns the file as a download.
pub async fn export_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(entity_type): Path<String>,
    Query(query): Query<ExportQuery>,
) -> Result<axum::response::Response, AppError> {
    query.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let entity_type = entity_type.to_lowercase();
    let format = query.format.unwrap_or_else(|| "xlsx".into());

    let data = DataIOService::export_entity(&pool, &entity_type, &format).await?;

    let content_type = DataIOService::content_type(&format);
    let ext = DataIOService::file_extension(&format);
    let disposition = format!("attachment; filename=\"{}.{}\"", entity_type, ext);

    let headers = [
        (header::CONTENT_TYPE, content_type),
        (header::CONTENT_DISPOSITION, &disposition),
    ];

    DataIOService::log_operation(
        &pool,
        None,
        None,
        "export",
        &entity_type,
        None,
        Some(serde_json::json!({"format": format}).to_string()),
        None,
    )
    .await?;

    Ok((headers, data).into_response())
}

/// GET `/api/v1/data/{entity_type}/template` — Download import template
///
/// Downloads a blank import template for a given entity type.
/// Logs the download operation.
pub async fn template_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(entity_type): Path<String>,
    Query(query): Query<ExportQuery>,
) -> Result<axum::response::Response, AppError> {
    query.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let entity_type = entity_type.to_lowercase();
    let format = query.format.unwrap_or_else(|| "xlsx".into());

    let data = DataIOService::download_template(&entity_type, &format).await?;

    let content_type = DataIOService::content_type(&format);
    let ext = DataIOService::file_extension(&format);
    let disposition = format!("attachment; filename=\"{}.{}_template.{}\"", entity_type, entity_type, ext);

    let headers = [
        (header::CONTENT_TYPE, content_type),
        (header::CONTENT_DISPOSITION, &disposition),
    ];

    DataIOService::log_operation(
        &pool,
        None,
        None,
        "download_template",
        &entity_type,
        None,
        None,
        None,
    )
    .await?;

    Ok((headers, data).into_response())
}

/// GET `/api/v1/data/logs` — Paginated operation logs
///
/// Returns paginated import/export operation logs for auditing.
pub async fn list_operation_logs_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<OperationLogQuery>,
) -> Result<Json<PaginatedResponse<OperationLog>>, AppError> {
    query.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

    let (items, total) = DataIOService::list_operation_logs(&pool, &query).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}
