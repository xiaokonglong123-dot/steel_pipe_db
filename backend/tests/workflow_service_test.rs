//! Workflow engine integration tests — definitions, conditional routing,
//! approve/reject state machine, task queries.

mod common;

use rust_decimal_macros::dec;
use steel_pipe_db::workflow::services::WorkflowService;

fn node(key: &str, assignee_value: &str, condition: Option<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "node_key": key,
        "assignee_type": "user",
        "assignee_value": assignee_value,
        "condition": condition,
    })
}

/// Seed the users referenced by tests (initiator=1, approvers=2/3).
/// The test pool runs migrations but NOT the app bootstrap, so users(id)
/// must be created explicitly for the workflow FK constraints.
async fn seed_users(pool: &sqlx::PgPool) {
    for (id, name) in [(1i64, "init"), (2, "manager"), (3, "director")] {
        let _ = sqlx::query(
            "INSERT INTO users (id, username, password_hash, display_name, role, tenant_id) \
             VALUES ($1, $2, 'x', $3, 'admin', 1) ON CONFLICT (id) DO NOTHING",
        )
        .bind(id)
        .bind(format!("user{}", id))
        .bind(name)
        .execute(pool)
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn create_definition_and_list() {
    let pool = common::test_pool().await;
    seed_users(&pool).await;
    let nodes = vec![node("approve_manager", "2", None), node("approve_director", "3", None)];
    let def = WorkflowService::create_definition(&pool, 1, "PO 审批", "purchase_order", None, &nodes, Some("approve_purchase_order"))
        .await
        .unwrap();
    assert!(def.id > 0);
    assert_eq!(def.entity_type, "purchase_order");

    let list = WorkflowService::list_definitions(&pool, 1, Some("purchase_order")).await.unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn empty_nodes_rejected() {
    let pool = common::test_pool().await;
    seed_users(&pool).await;
    let err = WorkflowService::create_definition(&pool, 1, "Empty", "x", None, &[], None).await;
    assert!(err.is_err(), "empty node list must be rejected");
}

#[tokio::test]
async fn start_instance_creates_pending_first_node() {
    let pool = common::test_pool().await;
    seed_users(&pool).await;
    let nodes = vec![node("approve_manager", "2", None), node("approve_director", "3", None)];
    let def = WorkflowService::create_definition(&pool, 1, "PO 审批", "purchase_order", None, &nodes, None)
        .await
        .unwrap();
    let inst = WorkflowService::start_instance(&pool, 1, def.id, "purchase_order", 42, None, 1)
        .await
        .unwrap();
    assert_eq!(inst.status, "running");
    assert_eq!(inst.current_step, 0);

    let tasks = WorkflowService::my_tasks(&pool, 2).await.unwrap();
    assert_eq!(tasks.len(), 1, "assignee user 2 must see the pending task");
    assert_eq!(tasks[0].node_key, "approve_manager");
}

#[tokio::test]
async fn conditional_routing_skips_high_amount_node() {
    let pool = common::test_pool().await;
    seed_users(&pool).await;
    // Node 2 only applies when amount > 50000.
    let nodes = vec![
        node("approve_manager", "2", None),
        node("approve_director", "3", Some(serde_json::json!({"amount_gt": 50000}))),
    ];
    let def = WorkflowService::create_definition(&pool, 1, "PO 金额条件", "purchase_order", None, &nodes, None)
        .await
        .unwrap();

    // Small amount: director node is skipped; approving manager finishes the flow.
    let inst = WorkflowService::start_instance(&pool, 1, def.id, "purchase_order", 43, Some(dec!(10000)), 1)
        .await
        .unwrap();
    let tasks = WorkflowService::my_tasks(&pool, 3).await.unwrap();
    assert!(tasks.is_empty(), "director must NOT receive a task for small amount");

    let done = WorkflowService::approve(&pool, WorkflowService::my_tasks(&pool, 2).await.unwrap()[0].id, 2, Some("ok")).await.unwrap();
    assert_eq!(done.status, "approved", "small amount flow finishes after manager approval");
}

#[tokio::test]
async fn large_amount_routes_to_director() {
    let pool = common::test_pool().await;
    seed_users(&pool).await;
    let nodes = vec![
        node("approve_manager", "2", None),
        node("approve_director", "3", Some(serde_json::json!({"amount_gt": 50000}))),
    ];
    let def = WorkflowService::create_definition(&pool, 1, "PO 金额条件", "purchase_order", None, &nodes, None)
        .await
        .unwrap();
    let inst = WorkflowService::start_instance(&pool, 1, def.id, "purchase_order", 44, Some(dec!(100000)), 1)
        .await
        .unwrap();
    assert_eq!(inst.status, "running");

    // Manager approves → director gets the task.
    let manager_task = WorkflowService::my_tasks(&pool, 2).await.unwrap().remove(0);
    let _ = WorkflowService::approve(&pool, manager_task.id, 2, None).await.unwrap();
    let director_tasks = WorkflowService::my_tasks(&pool, 3).await.unwrap();
    assert_eq!(director_tasks.len(), 1, "director must receive the task for large amount");
    assert_eq!(director_tasks[0].node_key, "approve_director");

    // Director approves → instance approved.
    let done = WorkflowService::approve(&pool, director_tasks[0].id, 3, Some("approved")).await.unwrap();
    assert_eq!(done.status, "approved");
}

#[tokio::test]
async fn reject_terminates_instance() {
    let pool = common::test_pool().await;
    seed_users(&pool).await;
    let nodes = vec![node("approve_manager", "2", None)];
    let def = WorkflowService::create_definition(&pool, 1, "PO 审批", "purchase_order", None, &nodes, None)
        .await
        .unwrap();
    WorkflowService::start_instance(&pool, 1, def.id, "purchase_order", 45, None, 1)
        .await
        .unwrap();
    let task = WorkflowService::my_tasks(&pool, 2).await.unwrap().remove(0);
    let inst = WorkflowService::reject(&pool, task.id, 2, "价格不合理").await.unwrap();
    assert_eq!(inst.status, "rejected");
}

#[tokio::test]
async fn non_assignee_cannot_approve() {
    let pool = common::test_pool().await;
    seed_users(&pool).await;
    let nodes = vec![node("approve_manager", "2", None)];
    let def = WorkflowService::create_definition(&pool, 1, "PO 审批", "purchase_order", None, &nodes, None)
        .await
        .unwrap();
    WorkflowService::start_instance(&pool, 1, def.id, "purchase_order", 46, None, 1)
        .await
        .unwrap();
    let task = WorkflowService::my_tasks(&pool, 2).await.unwrap().remove(0);
    let err = WorkflowService::approve(&pool, task.id, 3, None).await;
    assert!(err.is_err(), "non-assignee must not approve");
}

#[tokio::test]
async fn all_nodes_conditional_auto_approves() {
    let pool = common::test_pool().await;
    seed_users(&pool).await;
    let nodes = vec![node("approve_director", "3", Some(serde_json::json!({"amount_gt": 50000})))];
    let def = WorkflowService::create_definition(&pool, 1, "仅大额审批", "purchase_order", None, &nodes, None)
        .await
        .unwrap();
    let inst = WorkflowService::start_instance(&pool, 1, def.id, "purchase_order", 47, Some(dec!(1000)), 1)
        .await
        .unwrap();
    assert_eq!(inst.status, "approved", "no applicable node → auto-approved");
}

#[tokio::test]
async fn delegation_grants_task_visibility() {
    let pool = common::test_pool().await;
    seed_users(&pool).await;
    let nodes = vec![node("approve_manager", "2", None)];
    let def = WorkflowService::create_definition(&pool, 1, "PO 审批", "purchase_order", None, &nodes, None)
        .await
        .unwrap();
    WorkflowService::start_instance(&pool, 1, def.id, "purchase_order", 48, None, 1)
        .await
        .unwrap();

    // User 2 delegates to user 3.
    WorkflowService::delegate(&pool, 2, 3, None, 24).await.unwrap();
    let tasks = WorkflowService::my_tasks(&pool, 3).await.unwrap();
    assert_eq!(tasks.len(), 1, "delegatee must see the original assignee's tasks");
}
