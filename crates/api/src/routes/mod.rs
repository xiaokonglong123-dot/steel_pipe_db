//! 路由装配与共享设施。
//!
//! 拆分原则：一个领域一个文件；本文件只做 merge、共享失败出口与 OpenAPI 汇总。

pub mod audit;
pub mod auth;
pub mod catalog;
pub mod checks;
pub mod meta;
pub mod stats;
pub mod tubes;
pub mod users;

use axum::Router;
use axum::http::HeaderMap;
use utoipa::OpenApi;

use crate::error::{ApiError, ApiReject};
use crate::state::AppState;

pub const API_TAG: &str = "meta";
pub const AUTH_TAG: &str = "auth";
pub const USERS_TAG: &str = "users";
pub const TUBES_TAG: &str = "tubes";
pub const CATALOG_TAG: &str = "catalog";

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(meta::router())
        .merge(auth::router())
        .merge(audit::router())
        .merge(users::router())
        .merge(tubes::router())
        .merge(catalog::router())
        .merge(checks::router())
        .merge(stats::router())
}

/// 统一失败出口：带上请求头，让 401 能按 Accept 分流成登录页跳转（A11）。
pub(crate) fn fail(error: impl Into<ApiError>, headers: &HeaderMap) -> ApiReject {
    crate::error::reject(error.into(), headers)
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "SteelPipe API",
        version = env!("CARGO_PKG_VERSION"),
        description = "无缝钢管与筛管库存管理系统 API（单二进制 + SQLite）"
    ),
    paths(
        meta::health, meta::db_health, meta::openapi_json,
        auth::issue_csrf, auth::login, auth::refresh, auth::logout, auth::me,
        users::list_users, users::create_user, users::set_active, users::set_password,
        users::revoke_sessions,
        tubes::list_tubes, tubes::tube_detail, tubes::tube_history, tubes::stock_in,
        tubes::stock_out, tubes::batch_stock_out, tubes::transfer, tubes::set_quality,
        tubes::soft_delete, tubes::import_tubes,
        catalog::list_locations, catalog::create_location, catalog::update_location,
        catalog::set_location_active, catalog::list_standards, catalog::create_standard,
        catalog::update_standard, catalog::delete_standard,
        checks::list_checks, checks::create_check, checks::check_detail, checks::mark_item,
        checks::add_surplus, checks::remove_item, checks::confirm, checks::cancel, checks::recover,
        audit::list_audit, audit::verify_chain, stats::dashboard
    ),
    components(schemas(
        crate::ProblemDetails,
        meta::Health, meta::DbHealth,
        auth::CsrfResponse, auth::LoginRequest, auth::LoginResponse, auth::LogoutResponse,
        auth::IdentityDto,
        users::UserDto, users::UserListResponse, users::CreateUserRequest,
        users::SetActiveRequest, users::SetPasswordRequest, users::RevokeResponse,
        tubes::TubeDto, tubes::TubeListResponse, tubes::TubeDetailResponse,
        tubes::StockInRequest, tubes::StockInResponse, tubes::StockOutRequest,
        tubes::BatchStockOutRequest, tubes::BatchStockOutResponse, tubes::TransferRequest,
        tubes::TransferResponse, tubes::QualityRequest, tubes::HistoryEntryDto,
        tubes::ComplianceCheckDto, tubes::ImportResponse, tubes::ImportRowFailureDto,
        catalog::LocationDto, catalog::LocationListResponse, catalog::CreateLocationRequest,
        catalog::UpdateLocationRequest, catalog::SetLocationActiveRequest,
        catalog::StandardDto, catalog::StandardListResponse, catalog::StandardRequest,
        checks::CheckSessionDto, checks::CheckItemDto, checks::CheckDetailDto,
        checks::ExpectedTubeDto, checks::CheckListResponse, checks::CreateCheckRequest,
        checks::MarkItemRequest, checks::ConfirmResponse,
        audit::AuditQuery, audit::AuditEntryDto, audit::AuditListResponse,
        audit::VerifyRequest, audit::VerifyResponse,
        stats::DashboardDto, stats::TubeCountsDto, stats::LocationStatsDto,
        stats::TodayStatsDto, stats::AuditBriefDto, stats::AuditStatsDto
    ))
)]
pub struct ApiDoc;

pub fn build_openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
