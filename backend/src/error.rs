use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

// Re-export external error types for From impls
use argon2::password_hash::Error as PasswordHashError;

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
///
/// Use the `error_codes!` macro to define variants with their codes and HTTP status in one place.
macro_rules! error_codes {
    (
        $(
            $(#[$meta:meta])*
            $variant:ident($msg:literal) = ($code:expr, $status:expr)
        ),* $(,)?
    ) => {
        #[derive(Debug, thiserror::Error)]
        pub enum AppError {
            $(
                $(#[$meta])*
                #[error($msg)]
                $variant(String),
            )*
        }

        impl AppError {
            pub fn error_code(&self) -> u32 {
                match self {
                    $(Self::$variant(_) => $code),*
                }
            }

            pub fn status_code(&self) -> StatusCode {
                match self {
                    $(Self::$variant(_) => $status),*
                }
            }
        }
    };
}

error_codes! {
    /// Internal server error — unexpected condition that should not happen under normal operation.
    Internal("Internal server error: {0}") = (10001, StatusCode::INTERNAL_SERVER_ERROR),
    /// Validation failed — request payload didn't pass validation rules.
    Validation("Validation error: {0}") = (10002, StatusCode::BAD_REQUEST),
    /// Generic resource not found.
    NotFound("Resource not found: {0}") = (10003, StatusCode::NOT_FOUND),
    /// Bad request — the request is malformed or semantically invalid.
    BadRequest("Bad request: {0}") = (10004, StatusCode::BAD_REQUEST),

    /// Authentication failed — missing, invalid, or malformed credentials.
    Unauthorized("Unauthorized: {0}") = (11001, StatusCode::UNAUTHORIZED),
    /// JWT token has exceeded its expiry time.
    TokenExpired("Token expired") = (11002, StatusCode::UNAUTHORIZED),
    /// The user lacks the required role or permission.
    Forbidden("Forbidden: {0}") = (11003, StatusCode::FORBIDDEN),

    /// Item (商品) master record not found.
    ItemNotFound("Item not found: {0}") = (12001, StatusCode::NOT_FOUND),
    /// Item SKU already exists — duplicate detection.
    ItemSkuDuplicate("Item SKU already exists: {0}") = (12002, StatusCode::CONFLICT),
    /// Item status does not allow the requested operation.
    ItemStatusConflict("Item status conflict: {0}") = (12003, StatusCode::CONFLICT),

    /// Requested quantity exceeds available stock — ATP check failed.
    InsufficientStock("Insufficient stock") = (13001, StatusCode::CONFLICT),
    /// Warehouse location not found.
    LocationNotFound("Location not found: {0}") = (13002, StatusCode::NOT_FOUND),

    /// Order has reached a state where edits are no longer permitted.
    OrderCannotModify("Order cannot be modified: {0}") = (14001, StatusCode::CONFLICT),
    /// Order not found by the given order number or ID.
    OrderNotFound("Order not found: {0}") = (14002, StatusCode::NOT_FOUND),

    /// Supplier record not found.
    SupplierNotFound("Supplier not found: {0}") = (16001, StatusCode::NOT_FOUND),
    /// Supplier code violates the unique constraint.
    SupplierCodeDuplicate("Supplier code already exists: {0}") = (16002, StatusCode::CONFLICT),

    /// Customer record not found.
    CustomerNotFound("Customer not found: {0}") = (17001, StatusCode::NOT_FOUND),
    /// Customer code violates the unique constraint.
    CustomerCodeDuplicate("Customer code already exists: {0}") = (17002, StatusCode::CONFLICT),

    /// Bulk import failed — malformed file or row-level validation error.
    ImportError("Import error: {0}") = (18001, StatusCode::BAD_REQUEST),
    /// Export generation failed — data retrieval or file format error.
    ExportError("Export error: {0}") = (18002, StatusCode::BAD_REQUEST),

    /// Database-level failure (connection, constraint violation, or query error).
    Database("Database error: {0}") = (50001, StatusCode::INTERNAL_SERVER_ERROR),
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

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::Internal(format!("JSON serialization error: {}", err))
    }
}

impl From<PasswordHashError> for AppError {
    fn from(err: PasswordHashError) -> Self {
        Self::Internal(format!("Password hash error: {}", err))
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::Internal(format!("JWT error: {}", err))
    }
}
