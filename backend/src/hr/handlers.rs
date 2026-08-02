//! HR HTTP handlers — thin: extract → call service → respond.

use axum::extract::{Extension, Path, Query};
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;

use crate::dto::common::PaginationParams;
use crate::dto::hr_dto::{
    AttendanceQuery, CheckInRequest, CreateContractRequest, CreateEmployeeRequest,
    CreatePositionRequest, GenerateSalaryRequest, TerminateEmployeeRequest, UpdateEmployeeRequest,
};
use crate::error::AppError;
use crate::handlers::auth_handler::AuthenticatedUser;
use crate::hr::services::HrService;
use crate::models::hr::{
    HrAttendance, HrAttendanceRule, HrContract, HrEmployee, HrPosition, HrSalary,
};
use crate::response::{ApiResponse, PaginatedResponse};
#[derive(Debug, Deserialize)]
pub struct EmployeeFilter {
    pub department_id: Option<i64>,
    pub status: Option<String>,
    pub q: Option<String>,
}

// ---------------------------------------------------------------------------
// Employees
// ---------------------------------------------------------------------------

pub async fn list_employees(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Query(filter): Query<EmployeeFilter>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<HrEmployee>>, AppError> {
    let page = pagination.page.unwrap_or(1).max(1) as i64;
    let page_size = pagination.page_size.unwrap_or(20).clamp(1, 100) as i64;
    let (items, total) = HrService::list_employees(
        &pool,
        user.0.tenant_id,
        filter.department_id,
        filter.status.as_deref(),
        filter.q.as_deref(),
        page,
        page_size,
    )
    .await?;
    Ok(PaginatedResponse::ok(items, total as u64, page as u64, page_size as u64))
}

pub async fn get_employee(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<HrEmployee>>, AppError> {
    let item = HrService::get_employee(&pool, user.0.tenant_id, id).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn create_employee(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateEmployeeRequest>,
) -> Result<Json<ApiResponse<HrEmployee>>, AppError> {
    let item = HrService::create_employee(&pool, user.0.tenant_id, &payload).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn update_employee(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateEmployeeRequest>,
) -> Result<Json<ApiResponse<HrEmployee>>, AppError> {
    let item = HrService::update_employee(&pool, user.0.tenant_id, id, &payload).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn terminate_employee(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(payload): Json<TerminateEmployeeRequest>,
) -> Result<Json<ApiResponse<HrEmployee>>, AppError> {
    let item = HrService::terminate_employee(&pool, user.0.tenant_id, id, payload.reason.as_deref()).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn delete_employee(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    HrService::delete_employee(&pool, user.0.tenant_id, id).await?;
    Ok(ApiResponse::ok(()))
}

// ---------------------------------------------------------------------------
// Positions
// ---------------------------------------------------------------------------

pub async fn list_positions(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<HrPosition>>>, AppError> {
    let items = HrService::list_positions(&pool, user.0.tenant_id).await?;
    Ok(ApiResponse::ok(items))
}

pub async fn create_position(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(payload): Json<CreatePositionRequest>,
) -> Result<Json<ApiResponse<HrPosition>>, AppError> {
    let item = HrService::create_position(&pool, user.0.tenant_id, &payload).await?;
    Ok(ApiResponse::ok(item))
}

// ---------------------------------------------------------------------------
// Attendance
// ---------------------------------------------------------------------------

pub async fn list_attendance(
    Extension(pool): Extension<PgPool>,
    _user: AuthenticatedUser,
    Query(query): Query<AttendanceQuery>,
) -> Result<Json<ApiResponse<Vec<HrAttendance>>>, AppError> {
    let items = HrService::list_attendance(&pool, query.employee_id, query.from, query.to).await?;
    Ok(ApiResponse::ok(items))
}

pub async fn check_in(
    Extension(pool): Extension<PgPool>,
    _user: AuthenticatedUser,
    Json(payload): Json<CheckInRequest>,
) -> Result<Json<ApiResponse<HrAttendance>>, AppError> {
    let item = HrService::check_in(&pool, &payload).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn list_rules(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<HrAttendanceRule>>>, AppError> {
    let items = HrService::list_rules(&pool, user.0.tenant_id).await?;
    Ok(ApiResponse::ok(items))
}

// ---------------------------------------------------------------------------
// Salaries
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SalaryFilter {
    pub period: Option<String>,
}

pub async fn list_salaries(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Query(filter): Query<SalaryFilter>,
) -> Result<Json<ApiResponse<Vec<HrSalary>>>, AppError> {
    let items = HrService::list_salaries(&pool, user.0.tenant_id, filter.period.as_deref()).await?;
    Ok(ApiResponse::ok(items))
}

pub async fn get_salary(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<HrSalary>>, AppError> {
    let item = HrService::get_salary(&pool, user.0.tenant_id, id).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn generate_salaries(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(payload): Json<GenerateSalaryRequest>,
) -> Result<Json<ApiResponse<Vec<HrSalary>>>, AppError> {
    let items = HrService::generate_salaries(&pool, user.0.tenant_id, &payload.period).await?;
    Ok(ApiResponse::ok(items))
}

// ---------------------------------------------------------------------------
// Contracts
// ---------------------------------------------------------------------------

pub async fn create_contract(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateContractRequest>,
) -> Result<Json<ApiResponse<HrContract>>, AppError> {
    let item = HrService::create_contract(&pool, user.0.tenant_id, &payload).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn list_contracts(
    Extension(pool): Extension<PgPool>,
    _user: AuthenticatedUser,
    Path(employee_id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<HrContract>>>, AppError> {
    let items = HrService::list_contracts(&pool, employee_id).await?;
    Ok(ApiResponse::ok(items))
}
