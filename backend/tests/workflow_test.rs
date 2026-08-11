//! Workflow 集成测试 — 数据驱动审批流引擎：定义 CRUD + start_instance + transition
//! + task 查询/办结 + RBAC + 守卫。
//!
//! 注意：009_seed.sql 不含 workflow 种子数据（仅 roles/permissions），
//! 因此每个用例在 setup 阶段调 seed_purchase_order_workflow 自行插入一个
//! purchase_order 工作流定义（draft → submitted → approved/rejected，approved 为终态）。
//!
//! 结论（写报告用）：建议在 011_seed_workflows.sql 中补 purchase_order /
//! sales_order 的种子工作流，使 PO/SO submit 在 P0-11 无需人工预置即可走流。

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use sqlx::SqlitePool;
use tower::ServiceExt;

use common::test_pool;
use erp_v2::auth::bootstrap_admin;
use erp_v2::config::Config;
use erp_v2::http::router;
use erp_v2::middleware::auth::AuthUser;
use erp_v2::repos::workflow_repo;
use erp_v2::services::workflow_service;

struct TestServer {
    app: Router,
}

impl TestServer {
    async fn new(pool: SqlitePool) -> Self {
        let _cfg = Config {
            jwt_expiry_hours: 1,
            refresh_expiry_days: 1,
            ..Config::from_env().expect("config")
        };
        let secret = "test-secret".to_string();
        let app = router(pool, secret);
        Self { app }
    }

    async fn req(
        &self,
        method: &str,
        path: &str,
        body: String,
        auth: Option<&str>,
    ) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder().method(method).uri(path);
        if let Some(token) = auth {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
        let req = req
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();
        let resp = self.app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json = serde_json::from_slice(&body)
            .unwrap_or(serde_json::json!({ "raw": String::from_utf8_lossy(&body) }));
        (status, json)
    }
}

async fn admin_token(server: &TestServer) -> String {
    let body = r#"{"username":"admin","password":"admin123"}"#.to_string();
    let (_, json) = server.req("POST", "/auth/login", body, None).await;
    json["data"]["access_token"].as_str().unwrap().to_string()
}

async fn login_as(server: &TestServer, username: &str, password: &str) -> String {
    let body = format!(r#"{{"username":"{username}","password":"{password}"}}"#);
    let (_, json) = server.req("POST", "/auth/login", body, None).await;
    json["data"]["access_token"].as_str().unwrap().to_string()
}

fn admin_user(id: i64) -> AuthUser {
    AuthUser {
        id,
        username: "admin".into(),
        display_name: "Admin".into(),
        permissions: vec![
            "item.read".into(),
            "item.write".into(),
            "stock.read".into(),
            "stock.write".into(),
            "order.read".into(),
            "order.write".into(),
            "order.approve".into(),
            "finance.read".into(),
            "finance.write".into(),
            "report.read".into(),
            "user.manage".into(),
        ],
    }
}

/// setup：bootstrap admin + 插入 purchase_order 工作流定义，返回 workflow_id。
/// 状态机：draft(is_initial, doc=0) → submitted(doc=1) → approved(is_final, doc=2)
/// 动作：draft --submit--> submitted --approve--> approved --reject--> rejected(doc=2)
async fn seed_purchase_order_workflow(pool: &SqlitePool) -> i64 {
    bootstrap_admin(pool, "admin", "admin123").await.unwrap();
    let wf_id = workflow_repo::insert_workflow(pool, "PO Approval", "purchase_order", 1)
        .await
        .unwrap();

    let draft_id = workflow_repo::insert_state(pool, wf_id, "draft", 0, 1, 0)
        .await
        .unwrap();
    let submitted_id = workflow_repo::insert_state(pool, wf_id, "submitted", 1, 0, 0)
        .await
        .unwrap();
    workflow_repo::insert_state(pool, wf_id, "approved", 2, 0, 1)
        .await
        .unwrap();

    workflow_repo::insert_transition(pool, wf_id, draft_id, submitted_id, "submit", None, 0)
        .await
        .unwrap();
    let approved_id = workflow_repo::find_state_by_key(pool, wf_id, "approved")
        .await
        .unwrap()
        .unwrap()
        .id;
    workflow_repo::insert_transition(pool, wf_id, submitted_id, approved_id, "approve", None, 0)
        .await
        .unwrap();
    wf_id
}

// —— Tests ——

#[tokio::test]
async fn start_instance_creates_active_instance_at_initial_state() {
    let (pool, _dir) = test_pool().await;
    seed_purchase_order_workflow(&pool).await;
    let admin = admin_user(1);

    // 模拟 P0-11 集成：PO submit 后由 service 调 start_instance
    let instance = workflow_service::start_instance(&pool, "purchase_order", 42, &admin)
        .await
        .unwrap();
    assert_eq!(instance.business_type, "purchase_order");
    assert_eq!(instance.business_id, 42);
    assert_eq!(instance.current_state, "draft");
    assert_eq!(instance.status, "active");

    // find_active_instance_for 应返回此 active 实例
    let found = workflow_service::find_active_instance_for(&pool, "purchase_order", 42)
        .await
        .unwrap();
    assert!(
        found.is_some(),
        "find_active_instance_for should return Some"
    );
    assert_eq!(found.unwrap().current_state, "draft");
}

#[tokio::test]
async fn transition_moves_instance_to_approved_and_completes() {
    let (pool, _dir) = test_pool().await;
    let wf_id = seed_purchase_order_workflow(&pool).await;
    let admin = admin_user(1);

    // 创建一条已提交实例（提交动作也可走 transition；但这里直接插实例 + 跳到 submitted）
    let submitted = workflow_repo::find_state_by_key(&pool, wf_id, "submitted")
        .await
        .unwrap()
        .unwrap();
    let instance_id =
        workflow_repo::insert_instance(&pool, wf_id, "purchase_order", 100, &submitted.state_key)
            .await
            .unwrap();
    workflow_repo::insert_task(&pool, instance_id, "submitted", None)
        .await
        .unwrap();

    // approve 动作 → 实例置 completed，current_state=approved
    let updated = workflow_service::transition(&pool, instance_id, "approve", &admin, None)
        .await
        .unwrap();
    assert_eq!(updated.status, "completed");
    assert_eq!(updated.current_state, "approved");

    // 当前 pending task 应已被办结
    let tasks = workflow_service::list_tasks_for_instance(&pool, instance_id)
        .await
        .unwrap();
    assert!(
        tasks.iter().all(|t| t.status == "completed"),
        "all tasks completed"
    );
    assert!(tasks.iter().any(|t| t.action.as_deref() == Some("approve")));
}

#[tokio::test]
async fn transition_invalid_action_returns_invalid_transition() {
    let (pool, _dir) = test_pool().await;
    let _wf_id = seed_purchase_order_workflow(&pool).await;
    let admin = admin_user(1);

    let instance = workflow_service::start_instance(&pool, "purchase_order", 7, &admin)
        .await
        .unwrap();

    // 当前 state='draft'，approve 不存在（应为 submit）→ InvalidTransition = 17002
    let err = workflow_service::transition(&pool, instance.id, "approve", &admin, None)
        .await
        .expect_err("should be invalid transition");
    assert_eq!(err.code, erp_v2::error::ErrorCode::InvalidTransition);
    assert_eq!(err.code.code(), 17002);
}

#[tokio::test]
async fn transition_on_completed_instance_returns_validation() {
    let (pool, _dir) = test_pool().await;
    let wf_id = seed_purchase_order_workflow(&pool).await;
    let admin = admin_user(1);

    let submitted = workflow_repo::find_state_by_key(&pool, wf_id, "submitted")
        .await
        .unwrap()
        .unwrap();
    let instance_id =
        workflow_repo::insert_instance(&pool, wf_id, "purchase_order", 9, &submitted.state_key)
            .await
            .unwrap();
    workflow_repo::insert_task(&pool, instance_id, "submitted", None)
        .await
        .unwrap();
    workflow_service::transition(&pool, instance_id, "approve", &admin, None)
        .await
        .unwrap();

    // 实例已 completed；再发起动作 → Validation 10002
    let err = workflow_service::transition(&pool, instance_id, "approve", &admin, None)
        .await
        .expect_err("completed instance should reject transition");
    assert_eq!(err.code, erp_v2::error::ErrorCode::Validation);
    assert_eq!(err.code.code(), 10002);
}

#[tokio::test]
async fn find_active_instance_returns_none_after_completion() {
    let (pool, _dir) = test_pool().await;
    let wf_id = seed_purchase_order_workflow(&pool).await;
    let admin = admin_user(1);

    let submitted = workflow_repo::find_state_by_key(&pool, wf_id, "submitted")
        .await
        .unwrap()
        .unwrap();
    let instance_id =
        workflow_repo::insert_instance(&pool, wf_id, "purchase_order", 11, &submitted.state_key)
            .await
            .unwrap();
    workflow_repo::insert_task(&pool, instance_id, "submitted", None)
        .await
        .unwrap();
    workflow_service::transition(&pool, instance_id, "approve", &admin, None)
        .await
        .unwrap();

    let found = workflow_service::find_active_instance_for(&pool, "purchase_order", 11)
        .await
        .unwrap();
    assert!(found.is_none(), "no active instance after completion");
}

#[tokio::test]
async fn list_my_tasks_returns_pending_tasks_for_admin() {
    let (pool, _dir) = test_pool().await;
    seed_purchase_order_workflow(&pool).await;
    let admin = admin_user(1);

    workflow_service::start_instance(&pool, "purchase_order", 22, &admin)
        .await
        .unwrap();
    workflow_service::start_instance(&pool, "purchase_order", 23, &admin)
        .await
        .unwrap();

    let tasks = workflow_service::list_my_tasks(&pool, &admin)
        .await
        .unwrap();
    assert!(tasks.len() >= 2, "admin 通用待办 >= 2: {}", tasks.len());
    assert!(tasks.iter().all(|t| t.status == "pending"));
}

#[tokio::test]
async fn workflow_definition_crud_admin() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool.clone()).await;
    let token = admin_token(&server).await;
    let admin = admin_user(1);

    // create
    let body = r#"{"name":"SO Approval","applies_to":"sales_order","is_active":true}"#.to_string();
    let (st, json) = server.req("POST", "/workflows", body, Some(&token)).await;
    assert_eq!(st, StatusCode::CREATED, "create workflow: {json}");
    let wf_id = json["data"]["id"].as_i64().unwrap();

    // get
    let (st, json) = server
        .req(
            "GET",
            &format!("/workflows/{wf_id}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(json["data"]["workflow"]["name"], "SO Approval");

    // list
    let (st, json) = server
        .req("GET", "/workflows", String::new(), Some(&token))
        .await;
    assert_eq!(st, StatusCode::OK);
    assert!(json["data"]["items"].as_array().unwrap().len() >= 1);

    // update
    let body =
        r#"{"name":"SO Approval v2","applies_to":"sales_order","is_active":true}"#.to_string();
    let (st, json) = server
        .req("PUT", &format!("/workflows/{wf_id}"), body, Some(&token))
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(json["data"]["name"], "SO Approval v2");

    // delete
    let (st, _) = server
        .req(
            "DELETE",
            &format!("/workflows/{wf_id}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);

    let _ = admin;
}

#[tokio::test]
async fn delete_workflow_with_running_instance_returns_validation() {
    let (pool, _dir) = test_pool().await;
    // 011 自动注入了 purchase_order workflow(wf_id=1) 等等，直接用它创建 instance
    let admin = admin_user(1);
    workflow_service::start_instance(&pool, "purchase_order", 88, &admin)
        .await
        .unwrap();
    // 找到刚才启动instance所用workflow id（就是active的purchase_order）
    let wf_id = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM workflows WHERE applies_to='purchase_order' AND is_active=1 ORDER BY id ASC LIMIT 1",
    ).fetch_one(&pool).await.unwrap();

    let err = workflow_service::delete_workflow(&pool, wf_id, &admin)
        .await
        .expect_err("should refuse delete with running instance");
    assert_eq!(err.code, erp_v2::error::ErrorCode::Validation);
    assert_eq!(err.code.code(), 10002);
}

#[tokio::test]
async fn rbac_warehouse_cannot_manage_workflow_definitions() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let admin = admin_token(&server).await;
    let body =
        r#"{"username":"wh","password":"pass1234","display_name":"WH","role_ids":[3]}"#.to_string();
    let (st, json) = server.req("POST", "/users", body, Some(&admin)).await;
    assert_eq!(st, StatusCode::CREATED, "create wh user: {json}");
    let wh_token = login_as(&server, "wh", "pass1234").await;

    let body = r#"{"name":"X","applies_to":"sales_order","is_active":true}"#.to_string();
    let (st, json) = server
        .req("POST", "/workflows", body, Some(&wh_token))
        .await;
    assert_eq!(st, StatusCode::FORBIDDEN, "wh create workflow: {json}");
    assert_eq!(json["code"], 11003);

    let (st, _) = server
        .req("GET", "/workflows", String::new(), Some(&wh_token))
        .await;
    assert_eq!(st, StatusCode::FORBIDDEN, "wh list workflows should be 403");
}

#[tokio::test]
async fn complete_task_endpoint_drives_transition_and_completes_task() {
    let (pool, _dir) = test_pool().await;
    let wf_id = seed_purchase_order_workflow(&pool).await;
    let server = TestServer::new(pool.clone()).await;
    let token = admin_token(&server).await;
    let admin = admin_user(1);

    // 实例停在 submitted（人工插实例 + pending task）
    let submitted = workflow_repo::find_state_by_key(&pool, wf_id, "submitted")
        .await
        .unwrap()
        .unwrap();
    let instance_id =
        workflow_repo::insert_instance(&pool, wf_id, "purchase_order", 55, &submitted.state_key)
            .await
            .unwrap();
    let task_id = workflow_repo::insert_task(&pool, instance_id, "submitted", None)
        .await
        .unwrap();

    // 查我的待办（admin token 通过 middleware，权限含 order.read）
    let (st, json) = server
        .req(
            "GET",
            "/workflow-tasks?mine=true",
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "list my tasks: {json}");
    let items = json["data"]["items"].as_array().unwrap();
    assert!(
        items.iter().any(|t| t["id"] == task_id),
        "task {task_id} in list: {json}"
    );

    // complete task via endpoint → 实例置 completed
    let body = format!(r#"{{"action":"approve","comment":"ok"}}"#);
    let (st, json) = server
        .req(
            "POST",
            &format!("/workflow-tasks/{task_id}/complete"),
            body,
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "complete task: {json}");
    assert_eq!(json["data"]["status"], "completed");
    assert_eq!(json["data"]["current_state"], "approved");

    let tasks = workflow_service::list_tasks_for_instance(&pool, instance_id)
        .await
        .unwrap();
    assert!(tasks.iter().all(|t| t.status == "completed"));
    let _ = admin;
}

#[tokio::test]
async fn start_instance_without_active_workflow_returns_17001() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let admin = admin_user(1);

    // 011_seed_workflows 注入了 purchase_order/sales_order，所以用真不存在的 workflow_name
    let err = workflow_service::start_instance(&pool, "nonexistent_workflow_xyz", 1, &admin)
        .await
        .expect_err("should be workflow not found");
    assert_eq!(err.code, erp_v2::error::ErrorCode::WorkflowNotFound);
    assert_eq!(err.code.code(), 17001);
}

#[tokio::test]
async fn list_instances_filter_by_status_and_business_type() {
    let (pool, _dir) = test_pool().await;
    let wf_id = seed_purchase_order_workflow(&pool).await;
    let admin = admin_user(1);

    // 两个实例：一个 active，一个 completed
    workflow_service::start_instance(&pool, "purchase_order", 201, &admin)
        .await
        .unwrap();
    let submitted = workflow_repo::find_state_by_key(&pool, wf_id, "submitted")
        .await
        .unwrap()
        .unwrap();
    let iid =
        workflow_repo::insert_instance(&pool, wf_id, "purchase_order", 202, &submitted.state_key)
            .await
            .unwrap();
    workflow_repo::insert_task(&pool, iid, "submitted", None)
        .await
        .unwrap();
    workflow_service::transition(&pool, iid, "approve", &admin, None)
        .await
        .unwrap();

    let active = workflow_service::list_instances(
        &pool,
        Some("purchase_order".into()),
        Some("active".into()),
    )
    .await
    .unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].business_id, 201);

    let completed = workflow_service::list_instances(&pool, None, Some("completed".into()))
        .await
        .unwrap();
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].business_id, 202);
}

#[tokio::test]
async fn invalid_applies_to_rejected_by_service() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let admin = admin_user(1);

    let dto = workflow_service::CreateWorkflowRequest {
        name: "bogus".into(),
        applies_to: "purchase_invoice".into(),
        is_active: true,
    };
    let err = workflow_service::create_workflow(&pool, &dto, &admin)
        .await
        .expect_err("invalid applies_to should be rejected");
    assert_eq!(err.code, erp_v2::error::ErrorCode::Validation);
}
