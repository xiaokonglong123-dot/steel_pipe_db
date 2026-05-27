use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct ApiErrorResponse {
    pub success: bool,
    pub code: u32,
    pub request_id: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Application-level errors with numeric codes (100xx–50001) and HTTP status mapping.
/// Each variant carries the information needed for the frontend to display localized messages.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Internal server error — unexpected condition that should not happen under normal operation.
    /// Records the underlying cause as a string message.
    #[error("Internal server error: {0}")]
    Internal(String),
    /// Validation failed — request payload didn't pass validation rules (e.g., missing required field).
    #[error("Validation error: {0}")]
    Validation(String),
    /// Bad request — the request is malformed or semantically invalid beyond validation.
    #[error("Bad request: {0}")]
    BadRequest(String),
    /// Generic resource not found — the requested entity does not exist (non-specific).
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Authentication failed — missing, invalid, or malformed credentials.
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    /// JWT token has exceeded its expiry time — client must re-authenticate.
    #[error("Token expired")]
    TokenExpired,
    /// The user lacks the required role or permission for this operation.
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// Seamless/screen pipe not found by the given identifier.
    #[error("Pipe not found: {0}")]
    PipeNotFound(String),
    /// Pipe number already exists — duplicate detection for unique pipe identifiers.
    #[error("Pipe number already exists: {0}")]
    PipeNumberDuplicate(String),
    /// Pipe status does not allow the requested operation (e.g., scrapped pipe cannot be transferred).
    #[error("Pipe status conflict: {0}")]
    PipeStatusConflict(String),

    /// Requested quantity exceeds available stock — ATP check failed.
    #[error("Insufficient stock")]
    InsufficientStock,
    /// Warehouse location not found or does not belong to the expected zone.
    #[error("Location not found: {0}")]
    LocationNotFound(String),

    /// Order has reached a state where edits are no longer permitted.
    #[error("Order cannot be modified: {0}")]
    OrderCannotModify(String),
    /// Order not found by the given order number or ID.
    #[error("Order not found: {0}")]
    OrderNotFound(String),

    /// Quality inspection certificate not found or has been revoked.
    #[error("Quality cert not found: {0}")]
    QualityCertNotFound(String),
    /// File attachment referenced by a quality record does not exist.
    #[error("Attachment not found: {0}")]
    AttachmentNotFound(String),

    /// Supplier record not found by the given code or ID.
    #[error("Supplier not found: {0}")]
    SupplierNotFound(String),
    /// Supplier code violates the unique constraint — duplicate detected.
    #[error("Supplier code already exists: {0}")]
    SupplierCodeDuplicate(String),

    /// Customer record not found by the given code or ID.
    #[error("Customer not found: {0}")]
    CustomerNotFound(String),
    /// Customer code violates the unique constraint — duplicate detected.
    #[error("Customer code already exists: {0}")]
    CustomerCodeDuplicate(String),

    /// Bulk import failed — malformed file or row-level validation error.
    #[error("Import error: {0}")]
    ImportError(String),
    /// Export generation failed — data retrieval or file format error.
    #[error("Export error: {0}")]
    ExportError(String),

    /// Database-level failure (connection, constraint violation, or query error).
    #[error("Database error: {0}")]
    Database(String),
}

impl AppError {
    pub fn error_code(&self) -> u32 {
        match self {
            Self::Internal(_) => 10001,
            Self::Validation(_) => 10002,
            Self::BadRequest(_) => 10004,
            Self::NotFound(_) => 10003,
            Self::Unauthorized(_) => 11001,
            Self::TokenExpired => 11002,
            Self::Forbidden(_) => 11003,
            Self::PipeNotFound(_) => 12001,
            Self::PipeNumberDuplicate(_) => 12002,
            Self::PipeStatusConflict(_) => 12003,
            Self::InsufficientStock => 13001,
            Self::LocationNotFound(_) => 13002,
            Self::OrderCannotModify(_) => 14001,
            Self::OrderNotFound(_) => 14002,
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
            Self::Validation(_) | Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_)
            | Self::PipeNotFound(_)
            | Self::LocationNotFound(_)
            | Self::OrderNotFound(_) => StatusCode::NOT_FOUND,
            Self::Unauthorized(_) | Self::TokenExpired => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::PipeNumberDuplicate(_) => StatusCode::CONFLICT,
            Self::PipeStatusConflict(_) => StatusCode::CONFLICT,
            Self::InsufficientStock => StatusCode::CONFLICT,
            Self::OrderCannotModify(_) => StatusCode::CONFLICT,
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
            success: false,
            code: self.error_code(),
            request_id: format!("req_{}", Uuid::new_v4()),
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
