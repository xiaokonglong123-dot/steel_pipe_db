//! Catalog HTTP handlers — 商品主数据 CRUD
//!
//! 对齐 http/auth.rs：`Extension(pool): Extension<SqlitePool>`，DTO 解析+校验后调 service。

use axum::extract::{Extension, Multipart, Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::error::{AppError, ErrorCode};
use crate::repos::catalog_repo::ItemFilter;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::catalog_service;

#[derive(Deserialize)]
pub struct CreateItemRequest {
    pub sku: String,
    pub name: String,
    pub category: Option<String>,
    pub unit: Option<String>,
    pub spec: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateItemRequest {
    pub sku: String,
    pub name: String,
    pub category: Option<String>,
    pub unit: Option<String>,
    pub spec: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct ItemFilterQuery {
    pub sku: Option<String>,
    pub name: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn create_item(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateItemRequest>,
) -> Result<impl IntoResponse, AppError> {
    let item = catalog_service::create_item(
        &pool,
        &req.sku,
        &req.name,
        req.category.as_deref(),
        req.unit.as_deref(),
        req.spec.as_deref(),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(item))))
}

pub async fn list_items(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<ItemFilterQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    let filter = ItemFilter {
        sku: q.sku.as_deref(),
        name: q.name.as_deref(),
        category: q.category.as_deref(),
        status: q.status.as_deref(),
    };
    let (rows, total) = catalog_service::list_items(&pool, &filter, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}

pub async fn get_item(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let item = catalog_service::get_item(&pool, id).await?;
    Ok(Json(ApiResponse::ok(item)))
}

pub async fn update_item(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateItemRequest>,
) -> Result<impl IntoResponse, AppError> {
    let status = req.status.as_deref().unwrap_or("draft");
    let item = catalog_service::update_item(
        &pool,
        id,
        &req.sku,
        &req.name,
        req.category.as_deref(),
        req.unit.as_deref(),
        req.spec.as_deref(),
        status,
    )
    .await?;
    Ok((StatusCode::OK, Json(ApiResponse::ok(item))))
}

pub async fn delete_item(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    catalog_service::delete_item(&pool, id).await?;
    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({ "id": id }))),
    ))
}

pub async fn list_categories(
    Extension(pool): Extension<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let cats = catalog_service::list_categories(&pool).await?;
    Ok(Json(ApiResponse::ok(cats)))
}

#[derive(Serialize)]
pub struct ImportReport {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub errors: Vec<ImportRowError>,
}

#[derive(Serialize)]
pub struct ImportRowError {
    pub row: usize,
    pub message: String,
}

pub async fn import_items_csv(
    Extension(pool): Extension<SqlitePool>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut csv_bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::new(ErrorCode::Validation, &format!("multipart 解析失败: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" || name == "csv" {
            csv_bytes = Some(field.bytes().await.map_err(|e| {
                AppError::new(ErrorCode::Validation, &format!("读取上传字节失败: {e}"))
            })?.to_vec());
            break;
        }
    }
    let csv_bytes = csv_bytes.ok_or_else(|| AppError::new(ErrorCode::Validation, "未提供 CSV 文件（字段名应为 file 或 csv）"))?;

    let cursor = std::io::Cursor::new(csv_bytes);
    let mut reader = csv::Reader::from_reader(cursor);
    let headers = reader
        .headers()
        .map_err(|e| AppError::new(ErrorCode::Validation, &format!("CSV 头解析失败: {e}")))?
        .iter()
        .map(|s| s.trim().to_lowercase())
        .collect::<Vec<_>>();

    fn pick<'a>(headers: &'a [String], aliases: &[&str]) -> Option<usize> {
        for alias in aliases {
            if let Some(idx) = headers.iter().position(|h| h == alias) {
                return Some(idx);
            }
        }
        None
    }
    let idx_sku = pick(&headers, &["sku", "code"]).ok_or_else(|| AppError::new(ErrorCode::Validation, "CSV 缺少 sku 列"))?;
    let idx_name = pick(&headers, &["name", "名称"]).ok_or_else(|| AppError::new(ErrorCode::Validation, "CSV 缺少 name 列"))?;
    let idx_category = pick(&headers, &["category", "分类"]);
    let idx_unit = pick(&headers, &["unit", "单位"]);
    let idx_spec = pick(&headers, &["spec", "specification", "规格"]);

    let mut total = 0usize;
    let mut succeeded = 0usize;
    let mut failed = 0usize;
    let mut errors = Vec::<ImportRowError>::new();

    for (row_i, record) in reader.records().enumerate() {
        let row_no = row_i + 2;
        total += 1;
        let record = match record {
            Ok(r) => r,
            Err(e) => {
                failed += 1;
                errors.push(ImportRowError { row: row_no, message: format!("行解析失败: {e}") });
                continue;
            }
        };
        let get = |idx: Option<usize>| -> Option<String> {
            idx.and_then(|i| record.get(i)).map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
        };
        let sku = get(Some(idx_sku));
        let name = get(Some(idx_name));
        let sku = match sku { Some(s) => s, None => { failed += 1; errors.push(ImportRowError { row: row_no, message: "sku 为空".into() }); continue; } };
        let name = match name { Some(s) => s, None => { failed += 1; errors.push(ImportRowError { row: row_no, message: "name 为空".into() }); continue; } };
        let category = get(idx_category);
        let unit = get(idx_unit);
        let spec = get(idx_spec);

        match catalog_service::create_item(&pool, &sku, &name, category.as_deref(), unit.as_deref(), spec.as_deref()).await {
            Ok(_) => succeeded += 1,
            Err(e) => { failed += 1; errors.push(ImportRowError { row: row_no, message: e.to_string() }); }
        }
    }

    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(ImportReport { total, succeeded, failed, errors })),
    ))
}
