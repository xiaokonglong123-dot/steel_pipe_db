//! workflow_repo.definition — workflows / states / transitions 定义层

use sqlx::Executor;
use sqlx::SqlitePool;
use crate::error::{AppError, ErrorCode};

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct WorkflowRow {
    pub id: i64,
    pub name: String,
    pub applies_to: String,
    pub is_active: i64,
    pub created_at: String,
}


#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct WorkflowStateRow {
    pub id: i64,
    pub workflow_id: i64,
    pub state_key: String,
    pub doc_status: i64,
    pub is_initial: i64,
    pub is_final: i64,
}


#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct WorkflowTransitionRow {
    pub id: i64,
    pub workflow_id: i64,
    pub from_state_id: i64,
    pub to_state_id: i64,
    pub action: String,
    pub required_role: Option<String>,
    pub is_auto: i64,
    pub amount_threshold: Option<String>,
}


// —— Workflow definition CRUD ——

pub async fn list_workflows(pool: &SqlitePool) -> Result<Vec<WorkflowRow>, AppError> {
    let rows = sqlx::query_as::<_, WorkflowRow>(
        "SELECT id, name, applies_to, is_active, created_at
         FROM workflows ORDER BY id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}


pub async fn find_workflow_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<WorkflowRow>, AppError> {
    let row = sqlx::query_as::<_, WorkflowRow>(
        "SELECT id, name, applies_to, is_active, created_at
         FROM workflows WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}


/// 按 business_type 取 active 工作流定义（applies_to=? AND is_active=1）。
/// 多条 active 时取最早创建的一条（ORDER BY id ASC LIMIT 1），保证确定性。
pub async fn find_active_workflow_by_type(
    pool: &SqlitePool,
    applies_to: &str,
) -> Result<Option<WorkflowRow>, AppError> {
    let row = sqlx::query_as::<_, WorkflowRow>(
        "SELECT id, name, applies_to, is_active, created_at
         FROM workflows WHERE applies_to = ? AND is_active = 1
         ORDER BY id ASC LIMIT 1",
    )
    .bind(applies_to)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}


pub async fn insert_workflow(
    pool: &SqlitePool,
    name: &str,
    applies_to: &str,
    is_active: i64,
) -> Result<i64, AppError> {
    let result =
        sqlx::query("INSERT INTO workflows (name, applies_to, is_active) VALUES (?, ?, ?)")
            .bind(name)
            .bind(applies_to)
            .bind(is_active)
            .execute(pool)
            .await?;
    Ok(result.last_insert_rowid())
}


pub async fn update_workflow(
    pool: &SqlitePool,
    id: i64,
    name: &str,
    applies_to: &str,
    is_active: i64,
) -> Result<(), AppError> {
    let result =
        sqlx::query("UPDATE workflows SET name = ?, applies_to = ?, is_active = ? WHERE id = ?")
            .bind(name)
            .bind(applies_to)
            .bind(is_active)
            .bind(id)
            .execute(pool)
            .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::WorkflowNotFound, "审批流未找到"));
    }
    Ok(())
}


pub async fn delete_workflow(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM workflows WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::WorkflowNotFound, "审批流未找到"));
    }
    Ok(())
}


/// 统计某工作流下尚未结束（status='active'）的实例数（用于 update/delete 守卫）。
pub async fn count_active_instances_for_workflow(
    pool: &SqlitePool,
    workflow_id: i64,
) -> Result<i64, AppError> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM workflow_instances
         WHERE workflow_id = ? AND status = 'active'",
    )
    .bind(workflow_id)
    .fetch_one(pool)
    .await?;
    Ok(count)
}


// —— States ——

pub async fn list_states_for_workflow(
    pool: &SqlitePool,
    workflow_id: i64,
) -> Result<Vec<WorkflowStateRow>, AppError> {
    let rows = sqlx::query_as::<_, WorkflowStateRow>(
        "SELECT id, workflow_id, state_key, doc_status, is_initial, is_final
         FROM workflow_states WHERE workflow_id = ? ORDER BY id",
    )
    .bind(workflow_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}


pub async fn find_initial_state(
    pool: &SqlitePool,
    workflow_id: i64,
) -> Result<Option<WorkflowStateRow>, AppError> {
    let row = sqlx::query_as::<_, WorkflowStateRow>(
        "SELECT id, workflow_id, state_key, doc_status, is_initial, is_final
         FROM workflow_states WHERE workflow_id = ? AND is_initial = 1
         ORDER BY id ASC LIMIT 1",
    )
    .bind(workflow_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}


pub async fn find_state_by_key(
    pool: &SqlitePool,
    workflow_id: i64,
    state_key: &str,
) -> Result<Option<WorkflowStateRow>, AppError> {
    let row = sqlx::query_as::<_, WorkflowStateRow>(
        "SELECT id, workflow_id, state_key, doc_status, is_initial, is_final
         FROM workflow_states WHERE workflow_id = ? AND state_key = ?",
    )
    .bind(workflow_id)
    .bind(state_key)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}


pub async fn find_state_by_id(
    pool: &SqlitePool,
    state_id: i64,
) -> Result<Option<WorkflowStateRow>, AppError> {
    let row = sqlx::query_as::<_, WorkflowStateRow>(
        "SELECT id, workflow_id, state_key, doc_status, is_initial, is_final
         FROM workflow_states WHERE id = ?",
    )
    .bind(state_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}


pub async fn insert_state<'e, E>(
    executor: E,
    workflow_id: i64,
    state_key: &str,
    doc_status: i64,
    is_initial: i64,
    is_final: i64,
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "INSERT INTO workflow_states
            (workflow_id, state_key, doc_status, is_initial, is_final)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(workflow_id)
    .bind(state_key)
    .bind(doc_status)
    .bind(is_initial)
    .bind(is_final)
    .execute(executor)
    .await?;
    Ok(result.last_insert_rowid())
}


// —— Transitions ——

pub async fn list_outgoing_transitions(
    pool: &SqlitePool,
    workflow_id: i64,
    from_state_id: i64,
) -> Result<Vec<WorkflowTransitionRow>, AppError> {
    let rows = sqlx::query_as::<_, WorkflowTransitionRow>(
        "SELECT id, workflow_id, from_state_id, to_state_id, action, required_role, is_auto, amount_threshold
         FROM workflow_transitions
         WHERE workflow_id = ? AND from_state_id = ? ORDER BY id",
    )
    .bind(workflow_id)
    .bind(from_state_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}


pub async fn find_transition(
    pool: &SqlitePool,
    workflow_id: i64,
    from_state_id: i64,
    action: &str,
) -> Result<Option<WorkflowTransitionRow>, AppError> {
    let row = sqlx::query_as::<_, WorkflowTransitionRow>(
        "SELECT id, workflow_id, from_state_id, to_state_id, action, required_role, is_auto, amount_threshold
         FROM workflow_transitions
         WHERE workflow_id = ? AND from_state_id = ? AND action = ?
         ORDER BY id ASC LIMIT 1",
    )
    .bind(workflow_id)
    .bind(from_state_id)
    .bind(action)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}


pub async fn list_transitions_by_action(
    pool: &SqlitePool,
    workflow_id: i64,
    from_state_id: i64,
    action: &str,
) -> Result<Vec<WorkflowTransitionRow>, AppError> {
    let rows = sqlx::query_as::<_, WorkflowTransitionRow>(
        "SELECT id, workflow_id, from_state_id, to_state_id, action, required_role, is_auto, amount_threshold
         FROM workflow_transitions
         WHERE workflow_id = ? AND from_state_id = ? AND action = ?
         ORDER BY id ASC",
    )
    .bind(workflow_id)
    .bind(from_state_id)
    .bind(action)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}


pub async fn insert_transition<'e, E>(
    executor: E,
    workflow_id: i64,
    from_state_id: i64,
    to_state_id: i64,
    action: &str,
    required_role: Option<&str>,
    is_auto: i64,
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "INSERT INTO workflow_transitions
            (workflow_id, from_state_id, to_state_id, action, required_role, is_auto)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(workflow_id)
    .bind(from_state_id)
    .bind(to_state_id)
    .bind(action)
    .bind(required_role)
    .bind(is_auto)
    .execute(executor)
    .await?;
    Ok(result.last_insert_rowid())
}
