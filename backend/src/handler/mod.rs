pub mod auth;
pub mod pipes;
pub mod inventory;
pub mod quality;
pub mod purchases;
pub mod sales;
pub mod data_io;
pub mod contracts;
pub mod reports;
pub mod labels;

use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub meta: Option<PageMeta>,
    pub request_id: String,
}

#[derive(Serialize)]
pub struct ApiListResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub meta: Option<PageMeta>,
    pub request_id: String,
}

#[derive(Serialize)]
pub struct PageMeta {
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub total_pages: i64,
}

pub fn ok_response<T: Serialize>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: true,
        data: Some(data),
        meta: None,
        request_id: String::new(),
    })
}

pub fn list_response<T: Serialize>(data: Vec<T>, total: i64, page: i64, page_size: i64) -> Json<ApiListResponse<T>> {
    let total_pages = if total > 0 { (total as f64 / page_size as f64).ceil() as i64 } else { 0 };
    Json(ApiListResponse {
        success: true,
        data,
        meta: Some(PageMeta { page, page_size, total, total_pages }),
        request_id: String::new(),
    })
}
