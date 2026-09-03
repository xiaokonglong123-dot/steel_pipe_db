//! workflow_repo.task — workflow_tasks 任务层

use sqlx::Executor;
use sqlx::SqlitePool;
use crate::error::{AppError, ErrorCode};


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
