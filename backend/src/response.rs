//! 统一响应形状（对齐 PRD §7.4 / detailed-design）：
//!   成功:    { "success": true, "request_id": "...", "data": T }
//!   分页:    { "success": true, "request_id": "...", "data": { "items": [...] }, "meta": {...} }
//!   创建 → 201（ApiResponse::created）
//!   删除 → 204（空 body）

use serde::Serialize;
use uuid::Uuid;

fn new_request_id() -> String {
    format!("req_{}", Uuid::new_v4().simple())
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub request_id: String,
    pub data: T,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub success: bool,
    pub request_id: String,
    pub data: PaginatedData<T>,
    pub meta: Meta,
}

#[derive(Debug, Serialize)]
pub struct PaginatedData<T: Serialize> {
    pub items: Vec<T>,
}

#[derive(Debug, Serialize)]
pub struct Meta {
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            request_id: new_request_id(),
            data,
        }
    }

    /// 创建成功 — handler 返回 (StatusCode::CREATED, Json(...))
    pub fn created(data: T) -> Self {
        Self {
            success: true,
            request_id: new_request_id(),
            data,
        }
    }
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn ok(items: Vec<T>, total: i64, page: i64, page_size: i64) -> Self {
        let total_pages = if page_size == 0 {
            0
        } else {
            (total + page_size - 1) / page_size
        };
        Self {
            success: true,
            request_id: new_request_id(),
            data: PaginatedData { items },
            meta: Meta {
                total,
                page,
                page_size,
                total_pages,
            },
        }
    }
}
