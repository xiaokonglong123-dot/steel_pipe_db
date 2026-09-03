//! workflow_repo.instance — workflow_instances 生命周期

use sqlx::Executor;
use sqlx::SqlitePool;
use crate::error::{AppError, ErrorCode};


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
