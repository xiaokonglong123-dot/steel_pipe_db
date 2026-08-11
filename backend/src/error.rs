//! AppError — 统一错误枚举 + IntoResponse
//!
//! 错误码分域（对齐 PRD §7.4 / detailed-design §6.5）：
//!   100xx 通用 / 110xx Auth / 120xx 商品 / 130xx 库存
//!   140xx 订单 / 150xx 往来单位 / 160xx 财务 / 170xx 审批流 / 180xx 报表 / 50001 Database
//!
//! 原则：不向客户端暴露原始 SQL 错误字符串；From<sqlx::Error> 一律转 50001。

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // 通用 100xx
    Internal,
    Validation,
    NotFound,
    StatusConflict,
    // Auth 110xx
    Unauthorized,
    TokenExpired,
    Forbidden,
    // Catalog 120xx
    ItemNotFound,
    ItemDuplicateSku,
    // Inventory 130xx
    InsufficientStock,
    LocationNotFound,
    CheckNotFound,
    CheckNotDraft,
    // Orders 140xx
    OrderCannotModify,
    OrderNotFound,
    // Parties 150xx
    SupplierNotFound,
    CustomerNotFound,
    // Finance 160xx
    JournalNotFound,
    AccountNotFound,
    UnbalancedJournal,
    InvoiceNotFound,
    InvoiceAlreadyPaid,
    // Workflow 170xx
    WorkflowNotFound,
    InvalidTransition,
    // Config / DB
    Config,
    Database,
}

impl ErrorCode {
    pub fn code(self) -> i32 {
        match self {
            Self::Internal => 10001,
            Self::Validation => 10002,
            Self::NotFound => 10003,
            Self::StatusConflict => 10004,
            Self::Unauthorized => 11001,
            Self::TokenExpired => 11002,
            Self::Forbidden => 11003,
            Self::ItemNotFound => 12001,
            Self::ItemDuplicateSku => 12002,
            Self::InsufficientStock => 13001,
            Self::LocationNotFound => 13002,
            Self::CheckNotFound => 13003,
            Self::CheckNotDraft => 13004,
            Self::OrderCannotModify => 14001,
            Self::OrderNotFound => 14002,
            Self::SupplierNotFound => 15001,
            Self::CustomerNotFound => 15002,
            Self::JournalNotFound => 16001,
            Self::UnbalancedJournal => 16002,
            Self::InvoiceNotFound => 16003,
            Self::InvoiceAlreadyPaid => 16004,
            Self::AccountNotFound => 16005,
            Self::WorkflowNotFound => 17001,
            Self::InvalidTransition => 17002,
            Self::Config => 90001,
            Self::Database => 50001,
        }
    }

    pub fn status(self) -> StatusCode {
        match self {
            Self::Validation
            | Self::StatusConflict
            | Self::UnbalancedJournal
            | Self::ItemDuplicateSku
            | Self::InsufficientStock
            | Self::InvalidTransition
            | Self::OrderCannotModify => StatusCode::BAD_REQUEST,
            Self::CheckNotDraft => StatusCode::BAD_REQUEST,
            Self::Unauthorized | Self::TokenExpired => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound
            | Self::ItemNotFound
            | Self::LocationNotFound
            | Self::CheckNotFound
            | Self::OrderNotFound
            | Self::SupplierNotFound
            | Self::CustomerNotFound
            | Self::WorkflowNotFound => StatusCode::NOT_FOUND,
            Self::JournalNotFound | Self::InvoiceNotFound | Self::AccountNotFound => {
                StatusCode::NOT_FOUND
            }
            Self::InvoiceAlreadyPaid => StatusCode::BAD_REQUEST,
            Self::Internal | Self::Config | Self::Database => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn default_message(self) -> &'static str {
        match self {
            Self::Internal => "内部错误",
            Self::Validation => "请求参数校验失败",
            Self::NotFound => "资源未找到",
            Self::StatusConflict => "状态冲突",
            Self::Unauthorized => "未认证",
            Self::TokenExpired => "登录已过期，请重新登录",
            Self::Forbidden => "权限不足",
            Self::ItemNotFound => "商品未找到",
            Self::ItemDuplicateSku => "SKU 重复",
            Self::InsufficientStock => "库存不足",
            Self::LocationNotFound => "库位未找到",
            Self::CheckNotFound => "盘点单未找到",
            Self::CheckNotDraft => "盘点单当前状态不可修改",
            Self::OrderCannotModify => "订单当前状态不可修改",
            Self::OrderNotFound => "订单未找到",
            Self::SupplierNotFound => "供应商未找到",
            Self::CustomerNotFound => "客户未找到",
            Self::JournalNotFound => "日记账未找到",
            Self::AccountNotFound => "会计科目未找到",
            Self::UnbalancedJournal => "日记账借贷不平衡",
            Self::InvoiceNotFound => "发票未找到",
            Self::InvoiceAlreadyPaid => "发票已支付",
            Self::WorkflowNotFound => "审批流未找到",
            Self::InvalidTransition => "无效的状态迁移",
            Self::Config => "服务配置错误",
            Self::Database => "数据库错误",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Validation, msg)
    }

    pub fn status_code(&self) -> (StatusCode, i32) {
        (self.code.status(), self.code.code())
    }

    pub fn user_message(&self) -> String {
        self.message.clone()
    }

    pub fn log_error(&self) {
        tracing::error!(code = self.code.code(), message = %self.message, "app_error");
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code.code(), self.message)
    }
}

impl std::error::Error for AppError {}

#[derive(Serialize)]
struct ErrorBody {
    success: bool,
    code: i32,
    request_id: String,
    message: String,
    details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_code();
        let body = ErrorBody {
            success: false,
            code,
            request_id: format!("req_{}", Uuid::new_v4().simple()),
            message: self.user_message(),
            details: None,
        };
        (status, Json(body)).into_response()
    }
}

// From<sqlx::Error> → 一律转 Database(50001)，不暴露 SQL 字符串
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!(error = %e, "database error");
        Self {
            code: ErrorCode::Database,
            message: ErrorCode::Database.default_message().to_string(),
        }
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::validation(format!("无效的整数参数: {e}"))
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(e: argon2::password_hash::Error) -> Self {
        tracing::error!(error = %e, "password hashing error");
        Self::new(ErrorCode::Internal, "密码处理错误")
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        tracing::error!(error = %e, "jwt error");
        Self::new(
            ErrorCode::Unauthorized,
            ErrorCode::Unauthorized.default_message().to_string(),
        )
    }
}
