//! Workflow engine services — definition management, instance instantiation,
//! conditional routing, approve/reject state machine, task queries.

use chrono::Utc;
use sqlx::PgPool;
use serde_json::Value;

use crate::error::AppError;
use crate::models::workflow::{
    ApprovalNode, DefinitionNode, WorkflowDefinition, WorkflowDelegation, WorkflowInstance,
};
use crate::workflow::repos::{ApprovalNodeRepo, WorkflowDefinitionRepo, WorkflowDelegationRepo, WorkflowInstanceRepo};

pub struct WorkflowService;

impl WorkflowService {
    // -----------------------------------------------------------------------
    // Definitions
    // -----------------------------------------------------------------------

    pub async fn list_definitions(
        pool: &PgPool,
        tenant_id: i64,
        entity_type: Option<&str>,
    ) -> Result<Vec<WorkflowDefinition>, AppError> {
        WorkflowDefinitionRepo::list(pool, tenant_id, entity_type)
            .await
            .map_err(AppError::from)
    }

    pub async fn get_definition(
        pool: &PgPool,
        tenant_id: i64,
        id: i64,
    ) -> Result<WorkflowDefinition, AppError> {
        WorkflowDefinitionRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Workflow definition not found: {}", id)))
    }

    /// Parse the definition's `nodes` array into typed DefinitionNodes.
    fn parse_nodes(value: &Value) -> Result<Vec<DefinitionNode>, AppError> {
        let arr = value
            .as_array()
            .ok_or_else(|| AppError::Validation("definition_json.nodes must be an array".into()))?;
        let mut nodes = Vec::with_capacity(arr.len());
        for v in arr {
            nodes.push(serde_json::from_value::<DefinitionNode>(v.clone()).map_err(|_| {
                AppError::Validation(format!(
                    "Invalid node definition: missing node_key/assignee_type/assignee_value"
                ))
            })?);
        }
        if nodes.is_empty() {
            return Err(AppError::Validation("Workflow needs at least one node".into()));
        }
        Ok(nodes)
    }

    pub async fn create_definition(
        pool: &PgPool,
        tenant_id: i64,
        name: &str,
        entity_type: &str,
        description: Option<&str>,
        nodes: &[Value],
        callback_action: Option<&str>,
    ) -> Result<WorkflowDefinition, AppError> {
        if name.trim().is_empty() || name.len() > 200 {
            return Err(AppError::Validation("Name must be 1-200 characters".into()));
        }
        // Validate the node graph shape before persisting.
        Self::parse_nodes(&Value::Array(nodes.to_vec()))?;
        let def = WorkflowDefinitionRepo::create(
            pool,
            tenant_id,
            name,
            entity_type,
            description,
            &Value::Array(nodes.to_vec()),
            callback_action,
        )
        .await
        .map_err(AppError::from)?;
        Ok(def)
    }

    pub async fn update_definition(
        pool: &PgPool,
        tenant_id: i64,
        id: i64,
        name: Option<&str>,
        description: Option<&str>,
        nodes: Option<&[Value]>,
        is_active: Option<bool>,
    ) -> Result<WorkflowDefinition, AppError> {
        if let Some(n) = nodes {
            Self::parse_nodes(&Value::Array(n.to_vec()))?;
        }
        WorkflowDefinitionRepo::update(
            pool,
            tenant_id,
            id,
            name,
            description,
            nodes.map(|n| Value::Array(n.to_vec())).as_ref(),
            is_active,
        )
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Workflow definition not found: {}", id)))
    }

    pub async fn delete_definition(pool: &PgPool, tenant_id: i64, id: i64) -> Result<(), AppError> {
        let deleted = WorkflowDefinitionRepo::delete(pool, tenant_id, id)
            .await
            .map_err(AppError::from)?;
        if !deleted {
            return Err(AppError::NotFound(format!("Workflow definition not found: {}", id)));
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Instances
    // -----------------------------------------------------------------------

    /// Start a workflow: instantiate the definition's nodes (evaluating
    /// conditions up front so skipped nodes never enter the pending queue)
    /// and leave the first applicable node pending.
    pub async fn start_instance(
        pool: &PgPool,
        tenant_id: i64,
        definition_id: i64,
        entity_type: &str,
        entity_id: i64,
        amount: Option<rust_decimal::Decimal>,
        initiator_id: i64,
    ) -> Result<WorkflowInstance, AppError> {
        let def = Self::get_definition(pool, tenant_id, definition_id).await?;
        if !def.is_active {
            return Err(AppError::Validation("Workflow definition is inactive".into()));
        }
        let nodes = Self::parse_nodes(&def.definition_json)?;

        let mut tx = pool.begin().await.map_err(AppError::from)?;
        let instance = WorkflowInstanceRepo::create(
            &mut tx,
            def.id,
            tenant_id,
            entity_type,
            entity_id,
            amount,
            initiator_id,
        )
        .await
        .map_err(AppError::from)?;

        let mut pending_step: Option<(i32, usize)> = None;
        for (idx, node) in nodes.iter().enumerate() {
            let step = idx as i32;
            let created = ApprovalNodeRepo::insert(
                &mut tx,
                instance.id,
                step,
                &node.node_key,
                &node.assignee_type,
                node.assignee_value.as_deref(),
                node.condition.as_ref(),
            )
            .await
            .map_err(AppError::from)?;
            if Self::condition_skips(node, amount) {
                // Skipped nodes are recorded so the audit trail shows the
                // full definition even when branches are bypassed.
                ApprovalNodeRepo::skip(&mut tx, created.id)
                    .await
                    .map_err(AppError::from)?;
                continue;
            }
            if pending_step.is_none() {
                pending_step = Some((step, idx));
            }
        }

        let first_step = pending_step.map(|(s, _)| s).unwrap_or(0);
        WorkflowInstanceRepo::advance_step(&mut tx, instance.id, first_step)
            .await
            .map_err(AppError::from)?;
        // No applicable node at all → auto-approve the instance.
        if pending_step.is_none() {
            WorkflowInstanceRepo::finish(&mut tx, instance.id, "approved")
                .await
                .map_err(AppError::from)?;
        }
        tx.commit().await.map_err(AppError::from)?;
        // Re-read: the in-memory `instance` was created before any
        // status updates in this transaction.
        Self::get_instance(pool, instance.id).await
    }

    /// Condition routing: a node is skipped when its condition is present and
    /// NOT satisfied. Supported conditions: `{"amount_gt": N}` / `{"amount_lte": N}`.
    fn condition_skips(node: &DefinitionNode, amount: Option<rust_decimal::Decimal>) -> bool {
        let Some(cond) = &node.condition else { return false };
        let Some(amount) = amount else { return true }; // no amount context → skip conditional nodes
        let amt = amount;
        if let Some(n) = cond.get("amount_gt").and_then(Value::as_i64) {
            return amt <= rust_decimal::Decimal::from(n);
        }
        if let Some(n) = cond.get("amount_lte").and_then(Value::as_i64) {
            return amt > rust_decimal::Decimal::from(n);
        }
        false
    }

    // -----------------------------------------------------------------------
    // Tasks
    // -----------------------------------------------------------------------

    /// Pending tasks visible to a user: direct assignments plus delegations
    /// granted to them.
    pub async fn my_tasks(pool: &PgPool, user_id: i64) -> Result<Vec<ApprovalNode>, AppError> {
        let delegations = sqlx::query_as::<_, WorkflowDelegation>(
            "SELECT id, original_user_id, delegated_user_id, entity_type, starts_at, ends_at, is_active, created_at \
             FROM workflow_delegations WHERE delegated_user_id = $1 AND is_active = TRUE \
             AND starts_at <= NOW() AND (ends_at IS NULL OR ends_at >= NOW())",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        // Direct tasks: assignee_value = user id. Plus delegated original owners.
        let mut task_ids: Vec<i64> = delegations.iter().map(|d| d.original_user_id).collect();
        task_ids.push(user_id);
        let mut tasks = Vec::new();
        for uid in task_ids {
            let t = ApprovalNodeRepo::pending_for_user(pool, uid, None).await.map_err(AppError::from)?;
            tasks.extend(t);
        }
        // Deduplicate by node id.
        let mut seen = std::collections::HashSet::new();
        tasks.retain(|t| seen.insert(t.id));
        Ok(tasks)
    }

    pub async fn get_task(pool: &PgPool, node_id: i64) -> Result<(ApprovalNode, WorkflowInstance), AppError> {
        let node = ApprovalNodeRepo::find_by_id(pool, node_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Task not found: {}", node_id)))?;
        let instance = WorkflowInstanceRepo::find_by_id(pool, node.instance_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", node.instance_id)))?;
        Ok((node, instance))
    }

    /// Approve the current node, then route to the next pending node or
    /// finish the instance as approved.
    pub async fn approve(
        pool: &PgPool,
        node_id: i64,
        approver_id: i64,
        reason: Option<&str>,
    ) -> Result<WorkflowInstance, AppError> {
        let (node, instance) = Self::get_task(pool, node_id).await?;
        if node.status != "pending" {
            return Err(AppError::Validation(format!("Task is not pending (status: {})", node.status)));
        }
        if node.assignee_value.as_deref() != Some(&approver_id.to_string()) {
            return Err(AppError::Forbidden("This task is not assigned to you".into()));
        }

        let mut tx = pool.begin().await.map_err(AppError::from)?;
        ApprovalNodeRepo::decide(&mut tx, node.id, "approved", approver_id, reason)
            .await
            .map_err(AppError::from)?;

        let nodes = ApprovalNodeRepo::list_for_instance(pool, instance.id)
            .await
            .map_err(AppError::from)?;
        let next = nodes
            .iter()
            .find(|n| n.step_index > node.step_index && n.status == "pending");
        match next {
            Some(next_node) => {
                WorkflowInstanceRepo::advance_step(&mut tx, instance.id, next_node.step_index)
                    .await
                    .map_err(AppError::from)?;
            }
            None => {
                WorkflowInstanceRepo::finish(&mut tx, instance.id, "approved")
                    .await
                    .map_err(AppError::from)?;
            }
        }
        tx.commit().await.map_err(AppError::from)?;
        Self::get_instance(pool, instance.id).await
    }

    /// Reject terminates the whole instance (no rework loop in v1).
    pub async fn reject(
        pool: &PgPool,
        node_id: i64,
        approver_id: i64,
        reason: &str,
    ) -> Result<WorkflowInstance, AppError> {
        let (node, instance) = Self::get_task(pool, node_id).await?;
        if node.status != "pending" {
            return Err(AppError::Validation(format!("Task is not pending (status: {})", node.status)));
        }
        if node.assignee_value.as_deref() != Some(&approver_id.to_string()) {
            return Err(AppError::Forbidden("This task is not assigned to you".into()));
        }

        let mut tx = pool.begin().await.map_err(AppError::from)?;
        ApprovalNodeRepo::decide(&mut tx, node.id, "rejected", approver_id, Some(reason))
            .await
            .map_err(AppError::from)?;
        WorkflowInstanceRepo::finish(&mut tx, instance.id, "rejected")
            .await
            .map_err(AppError::from)?;
        tx.commit().await.map_err(AppError::from)?;
        Self::get_instance(pool, instance.id).await
    }

    pub async fn delegate(
        pool: &PgPool,
        original_user_id: i64,
        delegated_user_id: i64,
        entity_type: Option<&str>,
        hours: i64,
    ) -> Result<WorkflowDelegation, AppError> {
        let starts_at = Utc::now();
        let ends_at = starts_at + chrono::Duration::hours(hours);
        WorkflowDelegationRepo::create(
            pool,
            original_user_id,
            delegated_user_id,
            entity_type,
            starts_at,
            Some(ends_at),
        )
        .await
        .map_err(AppError::from)
    }

    async fn get_instance(pool: &PgPool, id: i64) -> Result<WorkflowInstance, AppError> {
        WorkflowInstanceRepo::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", id)))
    }
}
