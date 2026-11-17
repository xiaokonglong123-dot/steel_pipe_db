//! 统一错误映射：RFC 9457 Problem Details（设计 §15）+ 401/302 双轨（A11）。

use axum::Json;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Serialize;
use steelpipe_service::{AuthServiceError, InfraError, LimitScope, UserError};

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

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    Unavailable(String),
    #[error("{0}")]
    Invalid(String),
    #[error("未认证")]
    Unauthorized,
    #[error("无权执行该操作")]
    Forbidden,
    #[error("{0}不存在")]
    NotFound(&'static str),
    #[error("{0}不存在")]
    NotFound2(String),
    #[error("{0}已存在")]
    Conflict(&'static str),
    #[error("尝试过于频繁")]
    TooManyRequests { retry_after_secs: i64 },
    #[error("{0}")]
    Unprocessable(String),
    #[error("{0}")]
    StateConflict(String),
    #[error(transparent)]
    Internal(#[from] anyhow_lite::BoxedError),
}

/// 极简的错误包装，避免 api 层引入 anyhow。
pub mod anyhow_lite {
    pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;
}

impl ApiError {
    pub fn unavailable(detail: impl Into<String>) -> Self {
        Self::Unavailable(detail.into())
    }

    pub fn invalid(detail: impl Into<String>) -> Self {
        Self::Invalid(detail.into())
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Invalid(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound(_) | Self::NotFound2(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::TooManyRequests { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::Unprocessable(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::StateConflict(_) => StatusCode::CONFLICT,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn title(&self) -> &'static str {
        match self {
            Self::Unavailable(_) => "service-unavailable",
            Self::Invalid(_) => "invalid-request",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound(_) | Self::NotFound2(_) => "not-found",
            Self::Conflict(_) => "conflict",
            Self::TooManyRequests { .. } => "too-many-requests",
            Self::Unprocessable(_) => "unprocessable",
            Self::StateConflict(_) => "state-conflict",
            Self::Internal(_) => "internal-error",
        }
    }
}

impl From<AuthServiceError> for ApiError {
    fn from(error: AuthServiceError) -> Self {
        match error {
            AuthServiceError::InvalidCredentials => Self::Unauthorized,
            AuthServiceError::Unauthenticated => Self::Unauthorized,
            AuthServiceError::InvalidRefresh => Self::Unauthorized,
            AuthServiceError::Inactive => Self::Forbidden,
            AuthServiceError::RateLimited {
                scope,
                retry_after_secs,
            } => {
                tracing::debug!(?scope, retry_after_secs, "登录被限流");
                Self::TooManyRequests { retry_after_secs }
            }
            AuthServiceError::Infra(infra) => Self::from(infra),
            AuthServiceError::Auth(auth) => Self::Internal(Box::new(auth)),
        }
    }
}

impl From<crate::error::InfraError> for ApiError {
    fn from(error: crate::error::InfraError) -> Self {
        match error {
            crate::error::InfraError::NotFound(what) => Self::NotFound(what),
            crate::error::InfraError::Conflict(what) => Self::Conflict(what),
            other => Self::Internal(Box::new(other)),
        }
    }
}

impl From<steelpipe_service::InventoryError> for ApiError {
    fn from(error: steelpipe_service::InventoryError) -> Self {
        use steelpipe_service::InventoryError as E;
        match error {
            E::ComplianceRejected(checks) => {
                let detail = checks
                    .iter()
                    .filter(|check| !check.passed)
                    .map(|check| {
                        format!(
                            "{}: {}",
                            check.check_name,
                            check.reason.clone().unwrap_or_else(|| "未通过".into())
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                Self::Unprocessable(format!("API 5CT 合规校验未通过 —— {detail}"))
            }
            E::NoStandardSpec(grade) => {
                Self::Unprocessable(format!("没有钢级 {grade} 的标准规格，无法校验合规"))
            }
            E::ForceRequiresAdmin => Self::Forbidden,
            E::EmptyBatch => Self::Invalid("批量出库列表为空".into()),
            E::SameLocation => Self::Invalid("源库位与目标库位相同".into()),
            E::LocationNotFound(code) => Self::Invalid(format!("库位 {code} 不存在")),
            E::LocationInactive(code) => Self::Invalid(format!("库位 {code} 已停用")),
            E::TubeNotFound(pipe_id) => Self::Invalid(format!("管子 {pipe_id} 不存在或已删除")),
            E::Conflict(message) => Self::StateConflict(message),
            E::Domain(domain) => Self::StateConflict(domain.to_string()),
            E::Infra(infra) => Self::from(infra),
        }
    }
}

impl From<steelpipe_service::CatalogError> for ApiError {
    fn from(error: steelpipe_service::CatalogError) -> Self {
        use steelpipe_service::CatalogError as E;
        match error {
            E::NotFound(what) => Self::NotFound(what),
            E::Conflict(what) => Self::Conflict(what),
            E::Invalid(message) => Self::Invalid(message),
            E::Infra(infra) => Self::from(infra),
        }
    }
}

impl From<steelpipe_service::CheckError> for ApiError {
    fn from(error: steelpipe_service::CheckError) -> Self {
        use steelpipe_service::CheckError as E;
        match error {
            E::NotFound(id) => Self::NotFound2(format!("盘点单 {id}")),
            E::NotOpen => Self::StateConflict("盘点单不是 open 状态".into()),
            E::LocationUnavailable(code) => Self::Invalid(format!("库位 {code} 不存在或已停用")),
            E::TubeNotInScope(pipe_id) => {
                Self::Invalid(format!("管子 {pipe_id} 不在该盘点单范围内"))
            }
            E::InvalidMoved => Self::Invalid("moved 结论需要不同的实际位置".into()),
            E::NoStandardSpec(grade) => {
                Self::Unprocessable(format!("没有钢级 {grade} 的标准规格，无法校验盘盈合规"))
            }
            E::ComplianceRejected(checks) => {
                let detail = checks
                    .iter()
                    .map(|check| {
                        format!(
                            "{}: {}",
                            check.check_name,
                            check.reason.clone().unwrap_or_else(|| "未通过".into())
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                Self::Unprocessable(format!("盘盈合规校验未通过 —— {detail}"))
            }
            E::Conflict(message) => Self::StateConflict(message),
            E::Domain(domain) => Self::StateConflict(domain.to_string()),
            E::Infra(infra) => Self::from(infra),
        }
    }
}

impl From<steelpipe_service::ImportError> for ApiError {
    fn from(error: steelpipe_service::ImportError) -> Self {
        use steelpipe_service::ImportError as E;
        match error {
            E::MissingColumns(columns) => Self::Invalid(format!("CSV 缺少列：{columns}")),
            E::TooManyRows(rows) => Self::Invalid(format!(
                "行数 {rows} 超过单次上限 {}，请分批导入",
                steelpipe_service::import::MAX_IMPORT_ROWS
            )),
            E::Parse(message) => Self::Invalid(format!("CSV 解析失败：{message}")),
            E::Inventory(inventory) => Self::from(inventory),
        }
    }
}

impl From<UserError> for ApiError {
    fn from(error: UserError) -> Self {
        match error {
            UserError::Invalid(message) => Self::Invalid(message),
            UserError::NotFound => Self::NotFound("用户"),
            UserError::Conflict => Self::Conflict("用户名"),
            UserError::LastAdmin => Self::Invalid("不能移除最后一个管理员".into()),
            UserError::Infra(infra) => Self::from(infra),
            UserError::Auth(auth) => Self::Internal(Box::new(auth)),
        }
    }
}

/// 浏览器导航（`Accept: text/html`）落到登录页，其余走 JSON（A11）。
fn wants_html(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|accept| accept.contains("text/html"))
}

impl ApiError {
    fn render(&self, headers: &HeaderMap) -> Response {
        if matches!(self, Self::Unauthorized) && wants_html(headers) {
            return Redirect::to("/login").into_response();
        }

        let status = self.status();
        let mut response_headers = HeaderMap::new();
        if matches!(self, Self::Unauthorized) {
            response_headers.insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static("Bearer realm=\"steelpipe\""),
            );
        }
        if let Self::TooManyRequests { retry_after_secs } = self {
            if let Ok(value) = HeaderValue::from_str(&retry_after_secs.to_string()) {
                response_headers.insert(header::RETRY_AFTER, value);
            }
        }

        let body = ProblemDetails {
            kind: "about:blank".to_owned(),
            title: self.title().to_owned(),
            status: status.as_u16(),
            detail: Some(self.to_string()),
        };
        (status, response_headers, Json(body)).into_response()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        self.render(&HeaderMap::new())
    }
}

/// 已渲染的拒绝响应。装箱是为了让 `Result<Response, _>` 的 Err 变体足够小
/// （clippy::result_large_err），语义上它已经是最终响应，不需要再加工。
pub struct ApiReject(Box<Response>);

impl ApiReject {
    pub fn into_response(self) -> Response {
        *self.0
    }
}

impl From<ApiError> for ApiReject {
    /// 没有请求头上下文时退化为 JSON 响应（浏览器导航的 302 由 handler 显式调用 `fail`）。
    fn from(error: ApiError) -> Self {
        ApiReject(Box::new(error.render(&HeaderMap::new())))
    }
}

impl From<steelpipe_service::CatalogError> for ApiReject {
    fn from(error: steelpipe_service::CatalogError) -> Self {
        ApiError::from(error).into()
    }
}

impl From<steelpipe_service::InventoryError> for ApiReject {
    fn from(error: steelpipe_service::InventoryError) -> Self {
        ApiError::from(error).into()
    }
}

impl From<steelpipe_service::CheckError> for ApiReject {
    fn from(error: steelpipe_service::CheckError) -> Self {
        ApiError::from(error).into()
    }
}

impl From<steelpipe_service::ImportError> for ApiReject {
    fn from(error: steelpipe_service::ImportError) -> Self {
        ApiError::from(error).into()
    }
}

impl IntoResponse for ApiReject {
    fn into_response(self) -> Response {
        *self.0
    }
}

/// 供 handler 使用：把 `ApiError` 与请求头一起渲染成拒绝响应。
pub fn reject(error: ApiError, headers: &HeaderMap) -> ApiReject {
    ApiReject(Box::new(error.render(headers)))
}

/// limit scope 的可读名（用于日志与测试断言）。
pub fn limit_scope_name(scope: LimitScope) -> &'static str {
    match scope {
        LimitScope::Ip => "ip",
        LimitScope::Username => "username",
    }
}
