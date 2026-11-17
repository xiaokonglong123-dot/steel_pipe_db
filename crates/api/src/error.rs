//! 统一错误映射：RFC 9457 Problem Details（设计 §15）。

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// RFC 9457 Problem Details 响应体。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProblemDetails {
    /// 稳定的问题类型 URI，客户端据此分支。
    #[serde(rename = "type")]
    pub kind: String,
    /// 简短、人类可读的标题。
    pub title: String,
    /// HTTP 状态码。
    pub status: u16,
    /// 本次错误的细节（可选）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// API 层错误。M1 起在此追加 Unauthorized / Forbidden / Conflict / NotFound。
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    Unavailable(String),
    #[error("{0}")]
    Invalid(String),
}

impl ApiError {
    pub fn unavailable(detail: impl Into<String>) -> Self {
        Self::Unavailable(detail.into())
    }

    pub fn invalid(detail: impl Into<String>) -> Self {
        Self::Invalid(detail.into())
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Invalid(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn title(&self) -> &'static str {
        match self {
            Self::Unavailable(_) => "service-unavailable",
            Self::Invalid(_) => "invalid-request",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = ProblemDetails {
            kind: "about:blank".to_owned(),
            title: self.title().to_owned(),
            status: status.as_u16(),
            detail: Some(self.to_string()),
        };
        (status, Json(body)).into_response()
    }
}
