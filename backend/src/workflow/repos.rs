//! Workflow repositories — pure SQL, static methods, soft-delete aware.
//! Follows the project repo convention (unit structs, `&PgPool`).

use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use crate::models::workflow::{ApprovalNode, WorkflowDefinition, WorkflowDelegation, WorkflowInstance};

pub struct WorkflowDefinitionRepo;

impl WorkflowDefinitionRepo {
    pub async fn list(pool: &PgPool, tenant_id: i64, entity_type: Option<&str>) -> Result<Vec<WorkflowDefinition>, sqlx::Error> {
        sqlx::query_as::<_, WorkflowDefinition>(
            "SELECT id, tenant_id, name, entity_type, description, definition_json, \
                    callback_action, version, is_active, created_at, updated_at, deleted_at \
             FROM workflow_definitions WHERE tenant_id = $1 AND deleted_at IS NULL \
             AND ($2::text IS NULL OR entity_type = $2) ORDER BY id",
        )
        .bind(tenant_id)
        .bind(entity_type)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, tenant_id: i64, id: i64) -> Result<Option<WorkflowDefinition>, sqlx::Error> {
        sqlx::query_as::<_, WorkflowDefinition>(
            "SELECT id, tenant_id, name, entity_type, description, definition_json, \
                    callback_action, version, is_active, created_at, updated_at, deleted_at \
             FROM workflow_definitions WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        tenant_id: i64,
        name: &str,
        entity_type: &str,
        description: Option<&str>,
        definition_json: &serde_json::Value,
        callback_action: Option<&str>,
    ) -> Result<WorkflowDefinition, sqlx::Error> {
        sqlx::query_as::<_, WorkflowDefinition>(
            "INSERT INTO workflow_definitions \
             (tenant_id, name, entity_type, description, definition_json, callback_action) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, tenant_id, name, entity_type, description, definition_json, \
                       callback_action, version, is_active, created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(name)
        .bind(entity_type)
        .bind(description)
        .bind(definition_json)
        .bind(callback_action)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        tenant_id: i64,
        id: i64,
        name: Option<&str>,
        description: Option<&str>,
        definition_json: Option<&serde_json::Value>,
        is_active: Option<bool>,
    ) -> Result<Option<WorkflowDefinition>, sqlx::Error> {
        sqlx::query_as::<_, WorkflowDefinition>(
            "UPDATE workflow_definitions SET \
               name = COALESCE($3, name), \
               description = COALESCE($4, description), \
               definition_json = COALESCE($5, definition_json), \
               is_active = COALESCE($6, is_active), \
               updated_at = NOW() \
             WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL \
             RETURNING id, tenant_id, name, entity_type, description, definition_json, \
                       callback_action, version, is_active, created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(definition_json)
        .bind(is_active)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, tenant_id: i64, id: i64) -> Result<bool, sqlx::Error> {
        sqlx::query(
            "UPDATE workflow_definitions SET deleted_at = NOW() \
             WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() > 0)
    }
}

pub struct WorkflowInstanceRepo;

impl WorkflowInstanceRepo {
    pub async fn create(
        tx: &mut Transaction<'_, Postgres>,
        definition_id: i64,
        tenant_id: i64,
        entity_type: &str,
        entity_id: i64,
        amount: Option<rust_decimal::Decimal>,
        initiator_id: i64,
    ) -> Result<WorkflowInstance, sqlx::Error> {
        sqlx::query_as::<_, WorkflowInstance>(
            "INSERT INTO workflow_instances \
             (definition_id, tenant_id, entity_type, entity_id, amount, initiator_id) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, definition_id, tenant_id, entity_type, entity_id, amount, status, \
                       current_step, initiator_id, created_at, updated_at, finished_at",
        )
        .bind(definition_id)
        .bind(tenant_id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(amount)
        .bind(initiator_id)
        .fetch_one(&mut **tx)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<WorkflowInstance>, sqlx::Error> {
        sqlx::query_as::<_, WorkflowInstance>(
            "SELECT id, definition_id, tenant_id, entity_type, entity_id, amount, status, \
                    current_step, initiator_id, created_at, updated_at, finished_at \
             FROM workflow_instances WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_entity(
        pool: &PgPool,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Option<WorkflowInstance>, sqlx::Error> {
        sqlx::query_as::<_, WorkflowInstance>(
            "SELECT id, definition_id, tenant_id, entity_type, entity_id, amount, status, \
                    current_step, initiator_id, created_at, updated_at, finished_at \
             FROM workflow_instances WHERE entity_type = $1 AND entity_id = $2 \
             ORDER BY id DESC LIMIT 1",
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn finish(
        tx: &mut Transaction<'_, Postgres>,
        id: i64,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE workflow_instances SET status = $2, finished_at = NOW(), updated_at = NOW() \
             WHERE id = $1",
        )
        .bind(id)
        .bind(status)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn advance_step(
        tx: &mut Transaction<'_, Postgres>,
        id: i64,
        step: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE workflow_instances SET current_step = $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .bind(step)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}

pub struct ApprovalNodeRepo;

impl ApprovalNodeRepo {
    pub async fn insert(
        tx: &mut Transaction<'_, Postgres>,
        instance_id: i64,
        step_index: i32,
        node_key: &str,
        assignee_type: &str,
        assignee_value: Option<&str>,
        condition_json: Option<&serde_json::Value>,
    ) -> Result<ApprovalNode, sqlx::Error> {
        sqlx::query_as::<_, ApprovalNode>(
            "INSERT INTO approval_nodes \
             (instance_id, step_index, node_key, assignee_type, assignee_value, condition_json) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, instance_id, step_index, node_key, assignee_type, assignee_value, \
                       condition_json, status, approver_id, approval_reason, due_date, \
                       decided_at, created_at",
        )
        .bind(instance_id)
        .bind(step_index)
        .bind(node_key)
        .bind(assignee_type)
        .bind(assignee_value)
        .bind(condition_json)
        .fetch_one(&mut **tx)
        .await
    }

    pub async fn list_for_instance(
        pool: &PgPool,
        instance_id: i64,
    ) -> Result<Vec<ApprovalNode>, sqlx::Error> {
        sqlx::query_as::<_, ApprovalNode>(
            "SELECT id, instance_id, step_index, node_key, assignee_type, assignee_value, \
                    condition_json, status, approver_id, approval_reason, due_date, decided_at, created_at \
             FROM approval_nodes WHERE instance_id = $1 ORDER BY step_index",
        )
        .bind(instance_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<ApprovalNode>, sqlx::Error> {
        sqlx::query_as::<_, ApprovalNode>(
            "SELECT id, instance_id, step_index, node_key, assignee_type, assignee_value, \
                    condition_json, status, approver_id, approval_reason, due_date, decided_at, created_at \
             FROM approval_nodes WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Nodes pending approval for a user: direct user-assigned nodes plus
    /// role-assigned nodes matching the user's roles.
    pub async fn pending_for_user(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Vec<ApprovalNode>, sqlx::Error> {
        sqlx::query_as::<_, ApprovalNode>(
            "SELECT n.id, n.instance_id, n.step_index, n.node_key, n.assignee_type, \
                    n.assignee_value, n.condition_json, n.status, n.approver_id, \
                    n.approval_reason, n.due_date, n.decided_at, n.created_at \
             FROM approval_nodes n \
             WHERE n.status = 'pending' \
               AND ( \
                 (n.assignee_type = 'user' AND n.assignee_value = $1::text) \
                 OR (n.assignee_type = 'role' AND n.assignee_value IN ( \
                       SELECT r.name FROM user_roles ur \
                       JOIN roles r ON r.id = ur.role_id \
                       WHERE ur.user_id = $2 \
                     )) \
               ) \
             ORDER BY n.id DESC",
        )
        .bind(user_id.to_string())
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn decide(
        tx: &mut Transaction<'_, Postgres>,
        id: i64,
        status: &str,
        approver_id: i64,
        reason: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE approval_nodes SET status = $2, approver_id = $3, approval_reason = $4, \
                    decided_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .bind(status)
        .bind(approver_id)
        .bind(reason)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn skip(
        tx: &mut Transaction<'_, Postgres>,
        id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE approval_nodes SET status = 'skipped' WHERE id = $1")
            .bind(id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}

pub struct WorkflowDelegationRepo;

impl WorkflowDelegationRepo {
    pub async fn create(
        pool: &PgPool,
        original_user_id: i64,
        delegated_user_id: i64,
        entity_type: Option<&str>,
        starts_at: chrono::DateTime<Utc>,
        ends_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<WorkflowDelegation, sqlx::Error> {
        sqlx::query_as::<_, WorkflowDelegation>(
            "INSERT INTO workflow_delegations \
             (original_user_id, delegated_user_id, entity_type, starts_at, ends_at) \
             VALUES ($1, $2, $3, $4, $5) \
             RETURNING id, original_user_id, delegated_user_id, entity_type, starts_at, ends_at, is_active, created_at",
        )
        .bind(original_user_id)
        .bind(delegated_user_id)
        .bind(entity_type)
        .bind(starts_at)
        .bind(ends_at)
        .fetch_one(pool)
        .await
    }
}
