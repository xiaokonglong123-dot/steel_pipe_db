//! Sales 集成测试 — 销售订单 CRUD + ATP 预留 + 状态流转 + RBAC

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

async fn create_item(server: &TestServer, token: &str, sku: &str) -> i64 {
    let body = format!(r#"{{"sku":"{sku}","name":"n-{sku}"}}"#);
    let (st, json) = server.req("POST", "/items", body, Some(token)).await;
    assert_eq!(st, StatusCode::CREATED, "create item {sku}: {json}");
    json["data"]["id"].as_i64().unwrap()
}

async fn create_location(server: &TestServer, token: &str, code: &str) -> i64 {
    let body = format!(r#"{{"code":"{code}","name":"loc-{code}"}}"#);
    let (st, json) = server.req("POST", "/locations", body, Some(token)).await;
    assert_eq!(st, StatusCode::CREATED, "create location {code}: {json}");
    json["data"]["id"].as_i64().unwrap()
}

async fn create_customer(server: &TestServer, token: &str, code: &str) -> i64 {
    let body = format!(r#"{{"code":"{code}","name":"cust-{code}"}}"#);
    let (st, json) = server.req("POST", "/customers", body, Some(token)).await;
    assert_eq!(st, StatusCode::CREATED, "create customer {code}: {json}");
    json["data"]["id"].as_i64().unwrap()
}

async fn create_inbound(server: &TestServer, token: &str, items: &[(i64, i64, f64)]) -> i64 {
    let items_json: Vec<String> = items
        .iter()
        .map(|(i, l, q)| format!(r#"{{"item_id":{i},"location_id":{l},"quantity":{q}}}"#))
        .collect();
    let body = format!(
        r#"{{"inbound_type":"purchase","items":[{}]}}"#,
        items_json.join(",")
    );
    let (_, json) = server.req("POST", "/inbounds", body, Some(token)).await;
    json["data"]["id"].as_i64().unwrap()
}

async fn post_inbound(server: &TestServer, token: &str, id: i64) {
    server
        .req(
            "POST",
            &format!("/inbounds/{id}/post"),
            String::new(),
            Some(token),
        )
        .await;
}

async fn create_sales_order(
    server: &TestServer,
    token: &str,
    customer_id: i64,
    items: &[(i64, f64, &str)],
) -> (StatusCode, serde_json::Value) {
    let items_json: Vec<String> = items
        .iter()
        .map(|(i, q, p)| format!(r#"{{"item_id":{i},"quantity":{q},"unit_price":"{p}"}}"#))
        .collect();
    let body = format!(
        r#"{{"customer_id":{customer_id},"items":[{}]}}"#,
        items_json.join(",")
    );
    server.req("POST", "/sales-orders", body, Some(token)).await
}

// —— Tests ——

#[tokio::test]
async fn create_sales_order_with_two_items() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item1 = create_item(&server, &token, "SO-1").await;
    let item2 = create_item(&server, &token, "SO-2").await;
    let cust = create_customer(&server, &token, "CUST-1").await;

    let (st, json) = create_sales_order(
        &server,
        &token,
        cust,
        &[(item1, 3.0, "10.50"), (item2, 2.0, "20.00")],
    )
    .await;
    assert_eq!(st, StatusCode::CREATED, "create SO: {json}");
    assert_eq!(json["data"]["status"], "draft");
    assert_eq!(json["data"]["doc_status"], 0);
    assert!(json["data"]["order_no"].as_str().unwrap().starts_with("SO"));
}

#[tokio::test]
async fn total_amount_matches_sum() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item1 = create_item(&server, &token, "TOT-1").await;
    let item2 = create_item(&server, &token, "TOT-2").await;
    let cust = create_customer(&server, &token, "CUST-TOT").await;

    // 3 * 10.50 + 2 * 20.00 = 31.50 + 40.00 = 71.50
    let (_, json) = create_sales_order(
        &server,
        &token,
        cust,
        &[(item1, 3.0, "10.50"), (item2, 2.0, "20.00")],
    )
    .await;
    let total = json["data"]["total_amount"].as_str().unwrap();
    assert_eq!(total, "71.50", "total_amount: {json}");
}

#[tokio::test]
async fn list_filter_by_status_draft() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item = create_item(&server, &token, "LST-1").await;
    let loc = create_location(&server, &token, "LSTLOC-1").await;
    let cust = create_customer(&server, &token, "CUST-LST").await;

    let inbound_id = create_inbound(&server, &token, &[(item, loc, 10.0)]).await;
    post_inbound(&server, &token, inbound_id).await;

    let (_, j1) = create_sales_order(&server, &token, cust, &[(item, 1.0, "5.00")]).await;
    let id1 = j1["data"]["id"].as_i64().unwrap();
    let (_, _j2) = create_sales_order(&server, &token, cust, &[(item, 1.0, "5.00")]).await;

    let (st, _) = server
        .req(
            "POST",
            &format!("/sales-orders/{id1}/submit"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "submit for filter test");

    let (st, json) = server
        .req(
            "GET",
            "/sales-orders?status=draft&page=1&page_size=20",
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "list: {json}");
    let items = json["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "only one draft SO: {json}");
    assert_eq!(items[0]["status"], "draft");
    assert_eq!(json["meta"]["total"], 1);
}

#[tokio::test]
async fn create_with_nonexistent_customer_returns_404() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item = create_item(&server, &token, "NEC-1").await;
    let (st, json) = create_sales_order(&server, &token, 99999, &[(item, 1.0, "5.00")]).await;
    assert_eq!(st, StatusCode::NOT_FOUND, "nonexistent customer: {json}");
    // CustomerNotFound = 15002
    assert_eq!(json["code"], 15002);
}

#[tokio::test]
async fn create_with_nonexistent_item_returns_404() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let cust = create_customer(&server, &token, "CUST-NEI").await;
    let (st, json) = create_sales_order(&server, &token, cust, &[(99999, 1.0, "5.00")]).await;
    assert_eq!(st, StatusCode::NOT_FOUND, "nonexistent item: {json}");
    // ItemNotFound = 12001
    assert_eq!(json["code"], 12001);
}

#[tokio::test]
async fn atp_submit_succeeds_with_stock_and_creates_reservation() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item = create_item(&server, &token, "ATP-1").await;
    let loc = create_location(&server, &token, "ATPLOC-1").await;
    let cust = create_customer(&server, &token, "CUST-ATP").await;

    // 准备库存：入库 10 并过账
    let inbound_id = create_inbound(&server, &token, &[(item, loc, 10.0)]).await;
    post_inbound(&server, &token, inbound_id).await;

    // 创建 SO qty 8
    let (_, j) = create_sales_order(&server, &token, cust, &[(item, 8.0, "1.00")]).await;
    let so_id = j["data"]["id"].as_i64().unwrap();

    // 提交：ATP 通过（available 10 - 0 reserved = 10 >= 8）
    let (st, json) = server
        .req(
            "POST",
            &format!("/sales-orders/{so_id}/submit"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "submit ATP: {json}");
    assert_eq!(json["data"]["status"], "submitted");
    assert_eq!(json["data"]["doc_status"], 1);

    // 校验预留存在 status=active
    let (st, json) = server
        .req(
            "GET",
            &format!("/reservations?item_id={item}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "reservations: {json}");
    let reservations = json["data"]["items"].as_array().unwrap();
    assert_eq!(reservations.len(), 1, "one active reservation: {json}");
    assert_eq!(reservations[0]["status"], "active");
    assert_eq!(reservations[0]["quantity"], 8.0);
    assert_eq!(reservations[0]["order_id"], so_id);
    assert_eq!(reservations[0]["order_type"], "sales");
}

#[tokio::test]
async fn atp_submit_fails_when_reserved_exceeds_available() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item = create_item(&server, &token, "ATPF-1").await;
    let loc = create_location(&server, &token, "ATPFLOC-1").await;
    let cust = create_customer(&server, &token, "CUST-ATPF").await;

    // 库存 10
    let inbound_id = create_inbound(&server, &token, &[(item, loc, 10.0)]).await;
    post_inbound(&server, &token, inbound_id).await;

    // 第一单 qty 8 提交成功
    let (_, j1) = create_sales_order(&server, &token, cust, &[(item, 8.0, "1.00")]).await;
    let so1 = j1["data"]["id"].as_i64().unwrap();
    let (st, _) = server
        .req(
            "POST",
            &format!("/sales-orders/{so1}/submit"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);

    // 第二单 qty 5 提交失败（available = 10 - 8 reserved = 2 < 5）
    let (_, j2) = create_sales_order(&server, &token, cust, &[(item, 5.0, "1.00")]).await;
    let so2 = j2["data"]["id"].as_i64().unwrap();
    let (st, json) = server
        .req(
            "POST",
            &format!("/sales-orders/{so2}/submit"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "ATP fail: {json}");
    // InsufficientStock = 13001
    assert_eq!(json["code"], 13001);

    // 第二单仍为 draft
    let (st, json) = server
        .req(
            "GET",
            &format!("/sales-orders/{so2}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(
        json["data"]["order"]["status"], "draft",
        "SO stays draft on ATP fail: {json}"
    );
}

#[tokio::test]
async fn cancel_releases_reservations_and_restores_available() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item = create_item(&server, &token, "CAN-1").await;
    let loc = create_location(&server, &token, "CANLOC-1").await;
    let cust = create_customer(&server, &token, "CUST-CAN").await;

    // 库存 10
    let inbound_id = create_inbound(&server, &token, &[(item, loc, 10.0)]).await;
    post_inbound(&server, &token, inbound_id).await;

    // 提交 SO qty 8（预留 8，available 2）
    let (_, j) = create_sales_order(&server, &token, cust, &[(item, 8.0, "1.00")]).await;
    let so_id = j["data"]["id"].as_i64().unwrap();
    let (st, _) = server
        .req(
            "POST",
            &format!("/sales-orders/{so_id}/submit"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);

    // 取消：释放预留
    let (st, json) = server
        .req(
            "POST",
            &format!("/sales-orders/{so_id}/cancel"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "cancel: {json}");
    assert_eq!(json["data"]["status"], "cancelled");

    // 验证预留已释放
    let (st, json) = server
        .req(
            "GET",
            &format!("/reservations?item_id={item}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    let reservations = json["data"]["items"].as_array().unwrap();
    assert_eq!(
        reservations.len(),
        0,
        "no active reservations after cancel: {json}"
    );
}

#[tokio::test]
async fn after_cancel_new_submit_succeeds() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item = create_item(&server, &token, "RES-1").await;
    let loc = create_location(&server, &token, "RESLOC-1").await;
    let cust = create_customer(&server, &token, "CUST-RES").await;

    // 库存 10
    let inbound_id = create_inbound(&server, &token, &[(item, loc, 10.0)]).await;
    post_inbound(&server, &token, inbound_id).await;

    // 第一单 qty 8 提交成功
    let (_, j1) = create_sales_order(&server, &token, cust, &[(item, 8.0, "1.00")]).await;
    let so1 = j1["data"]["id"].as_i64().unwrap();
    let (st, _) = server
        .req(
            "POST",
            &format!("/sales-orders/{so1}/submit"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);

    // 取消第一单，available 恢复到 10
    let (st, _) = server
        .req(
            "POST",
            &format!("/sales-orders/{so1}/cancel"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);

    // 新单 qty 5 提交成功（available = 10 - 0 = 10 >= 5）
    let (_, j2) = create_sales_order(&server, &token, cust, &[(item, 5.0, "1.00")]).await;
    let so2 = j2["data"]["id"].as_i64().unwrap();
    let (st, json) = server
        .req(
            "POST",
            &format!("/sales-orders/{so2}/submit"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "submit after cancel: {json}");
    assert_eq!(json["data"]["status"], "submitted");
}

#[tokio::test]
async fn submit_twice_returns_conflict() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item = create_item(&server, &token, "TWICE-1").await;
    let loc = create_location(&server, &token, "TWICELOC-1").await;
    let cust = create_customer(&server, &token, "CUST-TWICE").await;

    let inbound_id = create_inbound(&server, &token, &[(item, loc, 10.0)]).await;
    post_inbound(&server, &token, inbound_id).await;

    let (_, j) = create_sales_order(&server, &token, cust, &[(item, 3.0, "1.00")]).await;
    let so_id = j["data"]["id"].as_i64().unwrap();

    let (st, _) = server
        .req(
            "POST",
            &format!("/sales-orders/{so_id}/submit"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);

    let (st, json) = server
        .req(
            "POST",
            &format!("/sales-orders/{so_id}/submit"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "second submit: {json}");
    // OrderCannotModify = 14001
    assert_eq!(json["code"], 14001);
}

#[tokio::test]
async fn approve_draft_returns_conflict() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item = create_item(&server, &token, "APD-1").await;
    let cust = create_customer(&server, &token, "CUST-APD").await;

    let (_, j) = create_sales_order(&server, &token, cust, &[(item, 1.0, "1.00")]).await;
    let so_id = j["data"]["id"].as_i64().unwrap();

    // 直接审批 draft 状态 -> 冲突
    let (st, json) = server
        .req(
            "POST",
            &format!("/sales-orders/{so_id}/approve"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "approve draft: {json}");
    assert_eq!(json["code"], 14001);
}

#[tokio::test]
async fn approve_submitted_sets_approved() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item = create_item(&server, &token, "APPS-1").await;
    let loc = create_location(&server, &token, "APPSLOC-1").await;
    let cust = create_customer(&server, &token, "CUST-APPS").await;

    let inbound_id = create_inbound(&server, &token, &[(item, loc, 10.0)]).await;
    post_inbound(&server, &token, inbound_id).await;

    let (_, j) = create_sales_order(&server, &token, cust, &[(item, 2.0, "1.00")]).await;
    let so_id = j["data"]["id"].as_i64().unwrap();

    // 提交
    let (st, _) = server
        .req(
            "POST",
            &format!("/sales-orders/{so_id}/submit"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);

    // 审批
    let (st, json) = server
        .req(
            "POST",
            &format!("/sales-orders/{so_id}/approve"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "approve: {json}");
    assert_eq!(json["data"]["status"], "approved");
    assert_eq!(json["data"]["doc_status"], 2);
}

#[tokio::test]
async fn reject_submitted_sets_rejected() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item = create_item(&server, &token, "REJ-1").await;
    let loc = create_location(&server, &token, "REJLOC-1").await;
    let cust = create_customer(&server, &token, "CUST-REJ").await;

    let inbound_id = create_inbound(&server, &token, &[(item, loc, 10.0)]).await;
    post_inbound(&server, &token, inbound_id).await;

    let (_, j) = create_sales_order(&server, &token, cust, &[(item, 2.0, "1.00")]).await;
    let so_id = j["data"]["id"].as_i64().unwrap();

    // 提交
    let (st, _) = server
        .req(
            "POST",
            &format!("/sales-orders/{so_id}/submit"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);

    // 驳回
    let (st, json) = server
        .req(
            "POST",
            &format!("/sales-orders/{so_id}/reject"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "reject: {json}");
    assert_eq!(json["data"]["status"], "rejected");
}

#[tokio::test]
async fn rbac_warehouse_read_not_write() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let admin = admin_token(&server).await;

    // warehouse role = id=3 (has order.read, NOT order.write per seed)
    let body =
        r#"{"username":"wh","password":"pass1234","display_name":"Warehouse","role_ids":[3]}"#
            .to_string();
    let (st, json) = server.req("POST", "/users", body, Some(&admin)).await;
    assert_eq!(st, StatusCode::CREATED, "create warehouse user: {json}");
    let wh_token = login_as(&server, "wh", "pass1234").await;

    let cust = create_customer(&server, &admin, "CUST-RBAC").await;
    let item = create_item(&server, &admin, "RBAC-1").await;

    // GET /sales-orders -> 200 (order.read)
    let (st, _) = server
        .req(
            "GET",
            "/sales-orders?page=1&page_size=20",
            String::new(),
            Some(&wh_token),
        )
        .await;
    assert_eq!(
        st,
        StatusCode::OK,
        "warehouse GET sales-orders should be 200"
    );

    // POST /sales-orders -> 403 (no order.write)
    let (st, json) = create_sales_order(&server, &wh_token, cust, &[(item, 1.0, "1.00")]).await;
    assert_eq!(
        st,
        StatusCode::FORBIDDEN,
        "warehouse POST sales-orders: {json}"
    );
    assert_eq!(json["code"], 11003);
}
