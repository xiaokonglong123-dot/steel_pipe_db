//! Workflow HTTP handlers — thin: extract → call service → respond.
//! Reuses the `AuthenticatedUser` extractor from the legacy auth handler.

use axum::extract::{Extension, Path, Query};
use axum::Json;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::auth::services::IdentityService;
use crate::dto::workflow_dto::{
    ApproveTaskRequest, CreateDefinitionRequest, DelegateTaskRequest, RejectTaskRequest,
    StartInstanceRequest,
};
use crate::error::AppError;
use crate::handlers::auth_handler::AuthenticatedUser;
use crate::models::workflow::{ApprovalNode, WorkflowDefinition, WorkflowDelegation, WorkflowInstance};
use crate::response::ApiResponse;
use crate::workflow::services::WorkflowService;

#[derive(Debug, Deserialize)]
pub struct DefinitionFilter {
    pub entity_type: Option<String>,
}

pub async fn list_definitions(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Query(filter): Query<DefinitionFilter>,
) -> Result<Json<ApiResponse<Vec<WorkflowDefinition>>>, AppError> {
    let items = WorkflowService::list_definitions(&pool, user.0.tenant_id, filter.entity_type.as_deref()).await?;
    Ok(ApiResponse::ok(items))
}

pub async fn get_definition(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<WorkflowDefinition>>, AppError> {
    let item = WorkflowService::get_definition(&pool, user.0.tenant_id, id).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn create_definition(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateDefinitionRequest>,
) -> Result<Json<ApiResponse<WorkflowDefinition>>, AppError> {
    let item = WorkflowService::create_definition(
        &pool,
        user.0.tenant_id,
        &payload.name,
        &payload.entity_type,
        payload.description.as_deref(),
        &payload.nodes,
        payload.callback_action.as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(item))
}

pub async fn update_definition(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(payload): Json<CreateDefinitionRequest>,
) -> Result<Json<ApiResponse<WorkflowDefinition>>, AppError> {
    let item = WorkflowService::update_definition(
        &pool,
        user.0.tenant_id,
        id,
        Some(&payload.name),
        payload.description.as_deref(),
        Some(&payload.nodes),
        None,
    )
    .await?;
    Ok(ApiResponse::ok(item))
}

pub async fn delete_definition(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    WorkflowService::delete_definition(&pool, user.0.tenant_id, id).await?;
    Ok(ApiResponse::ok(()))
}

pub async fn start_instance(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Json(payload): Json<StartInstanceRequest>,
) -> Result<Json<ApiResponse<WorkflowInstance>>, AppError> {
    let item = WorkflowService::start_instance(
        &pool,
        user.0.tenant_id,
        payload.definition_id,
        &payload.entity_type,
        payload.entity_id,
        payload.amount,
        user.0.user_id,
    )
    .await?;
    Ok(ApiResponse::ok(item))
}

pub async fn my_tasks(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<ApprovalNode>>>, AppError> {
    let items = WorkflowService::my_tasks(&pool, user.0.user_id).await?;
    Ok(ApiResponse::ok(items))
}

/// Task detail: node + owning instance (entity_type/entity_id for the UI to link back).
pub async fn get_task(
    Extension(pool): Extension<SqlitePool>,
    _user: AuthenticatedUser,
    Path(node_id): Path<i64>,
) -> Result<Json<ApiResponse<TaskDetail>>, AppError> {
    let (node, instance) = WorkflowService::get_task(&pool, node_id).await?;
    Ok(ApiResponse::ok(TaskDetail { node, instance }))
}

#[derive(serde::Serialize)]
pub struct TaskDetail {
    pub node: ApprovalNode,
    pub instance: WorkflowInstance,
}

pub async fn approve_task(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(node_id): Path<i64>,
    Json(payload): Json<ApproveTaskRequest>,
) -> Result<Json<ApiResponse<WorkflowInstance>>, AppError> {
    let item = WorkflowService::approve(&pool, node_id, user.0.user_id, payload.reason.as_deref()).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn reject_task(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(node_id): Path<i64>,
    Json(payload): Json<RejectTaskRequest>,
) -> Result<Json<ApiResponse<WorkflowInstance>>, AppError> {
    let item = WorkflowService::reject(&pool, node_id, user.0.user_id, &payload.reason).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn delegate_task(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Json(payload): Json<DelegateTaskRequest>,
) -> Result<Json<ApiResponse<WorkflowDelegation>>, AppError> {
    // Only the task assignee (or an admin) may delegate.
    let (node, _) = WorkflowService::get_task(&pool, payload.node_id).await?;
    let is_admin = IdentityService::user_permission_keys(&pool, user.0.user_id)
        .await?
        .iter()
        .any(|k| k == "system.admin");
    if node.assignee_value.as_deref() != Some(&user.0.user_id.to_string()) && !is_admin {
        return Err(AppError::Forbidden("Only the assignee may delegate this task".into()));
    }
    let item = WorkflowService::delegate(
        &pool,
        user.0.user_id,
        payload.delegated_user_id,
        payload.entity_type.as_deref(),
        24,
    )
    .await?;
    Ok(ApiResponse::ok(item))
}
