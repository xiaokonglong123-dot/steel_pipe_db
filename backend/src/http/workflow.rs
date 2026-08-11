//! Workflow HTTP handlers — 审批流引擎（定义 CRUD + 实例/任务查询/流转）
//!
//! 路由分组（在 http/mod.rs 装配）：
//! - workflow_admin（user.manage）：工作流定义 CRUD
//! - workflow_view（order.read）：实例/任务 GET + task complete POST
//!
//! TODO P0-11: integrate PO/SO submit to call workflow_service::start_instance

use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::ApiResponse;
use crate::services::workflow_service;

// —— DTOs ——

#[derive(Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub applies_to: String,
    #[serde(default)]
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct UpdateWorkflowRequest {
    pub name: String,
    pub applies_to: String,
    #[serde(default)]
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct InstanceListQuery {
    pub business_type: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct TaskListQuery {
    #[serde(default)]
    pub mine: bool,
}

#[derive(Deserialize)]
pub struct CompleteTaskRequest {
    pub action: String,
    #[serde(default)]
    pub comment: Option<String>,
}

// —— Workflow definition CRUD（admin only：user.manage）——

pub async fn create_workflow(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Json(req): Json<CreateWorkflowRequest>,
) -> Result<impl IntoResponse, AppError> {
    let dto = workflow_service::CreateWorkflowRequest {
        name: req.name,
        applies_to: req.applies_to,
        is_active: req.is_active,
    };
    let row = workflow_service::create_workflow(&pool, &dto, &user).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::created(row))))
}

pub async fn list_workflows(
    Extension(pool): Extension<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let rows = workflow_service::list_workflows(&pool).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": rows }))))
}

pub async fn get_workflow(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let def = workflow_service::get_workflow_definition(&pool, id).await?;
    Ok(Json(ApiResponse::ok(def)))
}

pub async fn update_workflow(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdateWorkflowRequest>,
) -> Result<impl IntoResponse, AppError> {
    let dto = workflow_service::UpdateWorkflowRequest {
        name: req.name,
        applies_to: req.applies_to,
        is_active: req.is_active,
    };
    let row = workflow_service::update_workflow(&pool, id, &dto, &user).await?;
    Ok(Json(ApiResponse::ok(row)))
}

pub async fn delete_workflow(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    workflow_service::delete_workflow(&pool, id, &user).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "deleted": id }))))
}

// —— Instances / Tasks（任意已认证用户：order.read）——

pub async fn list_instances(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<InstanceListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let rows = workflow_service::list_instances(&pool, q.business_type, q.status).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": rows }))))
}

pub async fn get_instance(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let detail = workflow_service::get_instance_detail(&pool, id).await?;
    Ok(Json(ApiResponse::ok(detail)))
}

pub async fn list_tasks(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Query(q): Query<TaskListQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 目前任一已认证用户都只能查自己的待办（mine=true 或省略均为"我的待办"）
    if !q.mine {
        return Err(AppError::validation("task 查询仅支持 mine=true"));
    }
    let rows = workflow_service::list_my_tasks(&pool, &user).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "items": rows }))))
}

pub async fn complete_task(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Path(task_id): Path<i64>,
    Json(req): Json<CompleteTaskRequest>,
) -> Result<impl IntoResponse, AppError> {
    let instance =
        workflow_service::complete_task(&pool, task_id, &req.action, &user, req.comment).await?;
    Ok(Json(ApiResponse::ok(instance)))
}
