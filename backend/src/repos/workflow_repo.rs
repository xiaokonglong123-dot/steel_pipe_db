//! Workflow 数据访问 — workflows / workflow_states / workflow_transitions /
//! workflow_instances / workflow_tasks 表（008_workflow.sql，数据驱动审批流）。
//!
//! 纯 SQL（sqlx），无业务逻辑。事务控制由 service 层 `pool.begin()` 负责；
//! 本 repo 中参与事务的函数对 `sqlx::Executor` 泛型化。

use sqlx::{Executor, SqlitePool};

use crate::error::{AppError, ErrorCode};

// —— Row structs ——

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

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct WorkflowInstanceRow {
    pub id: i64,
    pub workflow_id: i64,
    pub business_type: String,
    pub business_id: i64,
    pub current_state: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct WorkflowTaskRow {
    pub id: i64,
    pub instance_id: i64,
    pub state_key: String,
    pub assignee_id: Option<i64>,
    pub status: String,
    pub action: Option<String>,
    pub comment: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
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

// —— Instances ——

pub async fn find_active_instance_for(
    pool: &SqlitePool,
    business_type: &str,
    business_id: i64,
) -> Result<Option<WorkflowInstanceRow>, AppError> {
    let row = sqlx::query_as::<_, WorkflowInstanceRow>(
        "SELECT id, workflow_id, business_type, business_id, current_state, status,
                created_at, updated_at
         FROM workflow_instances
         WHERE business_type = ? AND business_id = ? AND status = 'active'
         ORDER BY id DESC LIMIT 1",
    )
    .bind(business_type)
    .bind(business_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn find_instance_by_id(
    pool: &SqlitePool,
    instance_id: i64,
) -> Result<Option<WorkflowInstanceRow>, AppError> {
    let row = sqlx::query_as::<_, WorkflowInstanceRow>(
        "SELECT id, workflow_id, business_type, business_id, current_state, status,
                created_at, updated_at
         FROM workflow_instances WHERE id = ?",
    )
    .bind(instance_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn insert_instance<'e, E>(
    executor: E,
    workflow_id: i64,
    business_type: &str,
    business_id: i64,
    initial_state_key: &str,
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "INSERT INTO workflow_instances
            (workflow_id, business_type, business_id, current_state, status)
         VALUES (?, ?, ?, ?, 'active')",
    )
    .bind(workflow_id)
    .bind(business_type)
    .bind(business_id)
    .bind(initial_state_key)
    .execute(executor)
    .await?;
    Ok(result.last_insert_rowid())
}

/// 更新实例当前状态；status 传 Some 时一并更新，传 None 时保持原 status。
pub async fn update_instance_state<'e, E>(
    executor: E,
    instance_id: i64,
    new_state: &str,
    status: Option<&str>,
) -> Result<(), AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = if let Some(s) = status {
        sqlx::query(
            "UPDATE workflow_instances
             SET current_state = ?, status = ?, updated_at = datetime('now')
             WHERE id = ?",
        )
        .bind(new_state)
        .bind(s)
        .bind(instance_id)
        .execute(executor)
        .await?
    } else {
        sqlx::query(
            "UPDATE workflow_instances
             SET current_state = ?, updated_at = datetime('now')
             WHERE id = ?",
        )
        .bind(new_state)
        .bind(instance_id)
        .execute(executor)
        .await?
    };
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::NotFound, "审批实例未找到"));
    }
    Ok(())
}

pub async fn complete_instance(pool: &SqlitePool, instance_id: i64) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE workflow_instances
         SET status = 'completed', updated_at = datetime('now')
         WHERE id = ?",
    )
    .bind(instance_id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::NotFound, "审批实例未找到"));
    }
    Ok(())
}

pub async fn cancel_instance(pool: &SqlitePool, instance_id: i64) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE workflow_instances
         SET status = 'cancelled', updated_at = datetime('now')
         WHERE id = ?",
    )
    .bind(instance_id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::NotFound, "审批实例未找到"));
    }
    Ok(())
}

pub async fn list_instances(
    pool: &SqlitePool,
    business_type_filter: Option<&str>,
    status_filter: Option<&str>,
) -> Result<Vec<WorkflowInstanceRow>, AppError> {
    let mut sql = String::from(
        "SELECT id, workflow_id, business_type, business_id, current_state, status,
                created_at, updated_at
         FROM workflow_instances WHERE 1 = 1",
    );
    if business_type_filter.is_some() {
        sql.push_str(" AND business_type = ?");
    }
    if status_filter.is_some() {
        sql.push_str(" AND status = ?");
    }
    sql.push_str(" ORDER BY id DESC");

    let mut q = sqlx::query_as::<_, WorkflowInstanceRow>(&sql);
    if let Some(v) = business_type_filter {
        q = q.bind(v);
    }
    if let Some(v) = status_filter {
        q = q.bind(v);
    }
    let rows = q.fetch_all(pool).await?;
    Ok(rows)
}

// —— Tasks ——

pub async fn insert_task<'e, E>(
    executor: E,
    instance_id: i64,
    state_key: &str,
    assignee_id: Option<i64>,
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "INSERT INTO workflow_tasks (instance_id, state_key, assignee_id, status)
         VALUES (?, ?, ?, 'pending')",
    )
    .bind(instance_id)
    .bind(state_key)
    .bind(assignee_id)
    .execute(executor)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn list_tasks_for_instance(
    pool: &SqlitePool,
    instance_id: i64,
) -> Result<Vec<WorkflowTaskRow>, AppError> {
    let rows = sqlx::query_as::<_, WorkflowTaskRow>(
        "SELECT id, instance_id, state_key, assignee_id, status, action, comment,
                created_at, completed_at
         FROM workflow_tasks WHERE instance_id = ? ORDER BY id",
    )
    .bind(instance_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 列出某用户的待办：assignee_id = user_id 或 assignee_id IS NULL（通用待办）。
pub async fn list_pending_tasks_for_user(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<Vec<WorkflowTaskRow>, AppError> {
    let rows = sqlx::query_as::<_, WorkflowTaskRow>(
        "SELECT id, instance_id, state_key, assignee_id, status, action, comment,
                created_at, completed_at
         FROM workflow_tasks
         WHERE status = 'pending'
           AND (assignee_id = ? OR assignee_id IS NULL)
         ORDER BY id DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 取某实例当前 pending 任务（取最早一条 pending，符合实际办理顺序）。泛型化以支持事务。
pub async fn find_pending_task_for_instance<'e, E>(
    executor: E,
    instance_id: i64,
) -> Result<Option<WorkflowTaskRow>, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let row = sqlx::query_as::<_, WorkflowTaskRow>(
        "SELECT id, instance_id, state_key, assignee_id, status, action, comment,
                created_at, completed_at
         FROM workflow_tasks
         WHERE instance_id = ? AND status = 'pending'
         ORDER BY id ASC LIMIT 1",
    )
    .bind(instance_id)
    .fetch_optional(executor)
    .await?;
    Ok(row)
}

/// 按 id 取任务（用于 complete 路径校验）。
pub async fn find_task_by_id(
    pool: &SqlitePool,
    task_id: i64,
) -> Result<Option<WorkflowTaskRow>, AppError> {
    let row = sqlx::query_as::<_, WorkflowTaskRow>(
        "SELECT id, instance_id, state_key, assignee_id, status, action, comment,
                created_at, completed_at
         FROM workflow_tasks WHERE id = ?",
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn complete_task<'e, E>(
    executor: E,
    task_id: i64,
    action: &str,
    comment: Option<&str>,
) -> Result<(), AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "UPDATE workflow_tasks
         SET status = 'completed', action = ?, comment = ?, completed_at = datetime('now')
         WHERE id = ? AND status = 'pending'",
    )
    .bind(action)
    .bind(comment)
    .bind(task_id)
    .execute(executor)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::NotFound, "任务未找到或已办结"));
    }
    Ok(())
}
