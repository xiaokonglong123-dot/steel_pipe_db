use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ApiErrorResponse {
    pub code: u32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // General (100xx)
    #[error("Internal server error: {0}")]
    Internal(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Resource not found: {0}")]
    NotFound(String),

    // Auth (110xx)
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Forbidden: {0}")]
    Forbidden(String),

    // Pipe (120xx)
    #[error("Pipe not found: {0}")]
    PipeNotFound(String),
    #[error("Pipe number already exists: {0}")]
    PipeNumberDuplicate(String),
    #[error("Pipe status conflict: {0}")]
    PipeStatusConflict(String),

    // Inventory (130xx)
    #[error("Insufficient stock")]
    InsufficientStock,
    #[error("Location not found: {0}")]
    LocationNotFound(String),
    #[error("Location is full")]
    LocationFull,

    // Orders (140xx)
    #[error("Order cannot be modified: {0}")]
    OrderCannotModify(String),
    #[error("Order not found: {0}")]
    OrderNotFound(String),
    #[error("Order not approved: {0}")]
    OrderNotApproved(String),

    // Quality (150xx)
    #[error("Quality cert not found: {0}")]
    QualityCertNotFound(String),
    #[error("Attachment not found: {0}")]
    AttachmentNotFound(String),

    // Supplier (160xx)
    #[error("Supplier not found: {0}")]
    SupplierNotFound(String),
    #[error("Supplier code already exists: {0}")]
    SupplierCodeDuplicate(String),

    // Customer (170xx)
    #[error("Customer not found: {0}")]
    CustomerNotFound(String),
    #[error("Customer code already exists: {0}")]
    CustomerCodeDuplicate(String),

    // Data IO (180xx)
    #[error("Import error: {0}")]
    ImportError(String),
    #[error("Export error: {0}")]
    ExportError(String),

    // Generic DB
    #[error("Database error: {0}")]
    Database(String),
}

impl AppError {
    pub fn error_code(&self) -> u32 {
        match self {
            Self::Internal(_) => 10001,
            Self::Validation(_) => 10002,
            Self::NotFound(_) => 10003,
            Self::Unauthorized(_) => 11001,
            Self::TokenExpired => 11002,
            Self::Forbidden(_) => 11003,
            Self::PipeNotFound(_) => 12001,
            Self::PipeNumberDuplicate(_) => 12002,
            Self::PipeStatusConflict(_) => 12003,
            Self::InsufficientStock => 13001,
            Self::LocationFull => 13002,
            Self::LocationNotFound(_) => 13003,
            Self::OrderCannotModify(_) => 14001,
            Self::OrderNotFound(_) => 14002,
            Self::OrderNotApproved(_) => 14003,
            Self::QualityCertNotFound(_) => 15001,
            Self::AttachmentNotFound(_) => 15002,
            Self::SupplierNotFound(_) => 16001,
            Self::SupplierCodeDuplicate(_) => 16002,
            Self::CustomerNotFound(_) => 17001,
            Self::CustomerCodeDuplicate(_) => 17002,
            Self::ImportError(_) => 18001,
            Self::ExportError(_) => 18002,
            Self::Database(_) => 50001,
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Internal(_) | Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_)
            | Self::PipeNotFound(_)
            | Self::LocationNotFound(_)
            | Self::OrderNotFound(_) => StatusCode::NOT_FOUND,
            Self::Unauthorized(_) | Self::TokenExpired => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::PipeNumberDuplicate(_) => StatusCode::CONFLICT,
            Self::PipeStatusConflict(_) => StatusCode::CONFLICT,
            Self::InsufficientStock => StatusCode::CONFLICT,
            Self::LocationFull => StatusCode::CONFLICT,
            Self::OrderCannotModify(_) => StatusCode::CONFLICT,
            Self::OrderNotApproved(_) => StatusCode::CONFLICT,
            Self::QualityCertNotFound(_) | Self::AttachmentNotFound(_) => StatusCode::NOT_FOUND,
            Self::SupplierNotFound(_) | Self::CustomerNotFound(_) => StatusCode::NOT_FOUND,
            Self::SupplierCodeDuplicate(_) | Self::CustomerCodeDuplicate(_) => StatusCode::CONFLICT,
            Self::ImportError(_) | Self::ExportError(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ApiErrorResponse {
            code: self.error_code(),
            message: self.to_string(),
            details: None,
        };
        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.to_string())
    }
}
