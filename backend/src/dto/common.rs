use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

impl PaginationParams {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn page_size(&self) -> u64 {
        self.page_size.unwrap_or(20).clamp(1, 100)
    }

    pub fn offset(&self) -> u64 {
        (self.page() - 1) * self.page_size()
    }

    pub fn sort_order_sql(&self) -> String {
        match self.sort_order.as_deref() {
            Some("asc") => "ASC".to_string(),
            Some("desc") => "DESC".to_string(),
            _ => "DESC".to_string(),
        }
    }
}
