use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

fn gen_request_id() -> String {
    format!("req_{}", Uuid::new_v4())
}

/// Wraps a single data payload with standard envelope fields.
/// `request_id` is auto-generated in each constructor — callers don't pass it.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub request_id: String,
    pub data: T,
}

/// Pagination metadata, mirrored on `PaginatedResponse` for convenience.
/// Frontend can read `meta` without unwrapping `data.items`.
#[derive(Debug, Serialize)]
pub struct Meta {
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize)]
pub struct PaginatedData<T: Serialize> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub success: bool,
    pub request_id: String,
    pub meta: Meta,
    pub data: PaginatedData<T>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Wrap a data payload in the standard API envelope. Returns 200 OK.
    pub fn ok(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            request_id: gen_request_id(),
            data,
        })
    }

    /// Wrap a data payload and return 201 Created — used for resource-creation endpoints.
    pub fn created(data: T) -> Response {
        (
            StatusCode::CREATED,
            Json(Self {
                success: true,
                request_id: gen_request_id(),
                data,
            }),
        )
            .into_response()
    }
}

impl<T: Serialize> PaginatedResponse<T> {
    /// Build a paginated response envelope with items and computed metadata. Returns 200 OK.
    pub fn ok(items: Vec<T>, total: u64, page: u64, page_size: u64) -> Json<Self> {
        let total_pages = if total == 0 {
            0
        } else {
            total.div_ceil(page_size)
        };
        Json(Self {
            success: true,
            request_id: gen_request_id(),
            meta: Meta {
                total,
                page,
                page_size,
                total_pages,
            },
            data: PaginatedData {
                items,
                total,
                page,
                page_size,
                total_pages,
            },
        })
    }
}

/// 204 No Content — for deletions or operations that return no body.
pub fn no_content() -> Response {
    StatusCode::NO_CONTENT.into_response()
}


