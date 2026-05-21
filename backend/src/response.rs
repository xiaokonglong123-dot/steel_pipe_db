use axum::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
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
    pub data: PaginatedData<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            data,
        })
    }
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn ok(items: Vec<T>, total: u64, page: u64, page_size: u64) -> Json<Self> {
        let total_pages = if total == 0 {
            0
        } else {
            (total + page_size - 1) / page_size
        };
        Json(Self {
            success: true,
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

#[derive(Debug, Serialize)]
pub struct ListParams {
    pub page: u64,
    pub page_size: u64,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub q: Option<String>,
}

impl Default for ListParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
            sort_by: None,
            sort_order: None,
            q: None,
        }
    }
}
