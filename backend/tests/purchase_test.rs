//! Purchase Orders 集成测试 — 采购订单 CRUD + 状态迁移 + 校验 + RBAC

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use std::str::FromStr;
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

async fn create_user(server: &TestServer, admin_token: &str, username: &str, role_ids: &[i64]) {
    let roles = role_ids
        .iter()
        .map(|r| r.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let body = format!(
        r#"{{"username":"{username}","password":"pass1234","display_name":"{username}","role_ids":[{roles}]}}"#
    );
    let (st, json) = server.req("POST", "/users", body, Some(admin_token)).await;
    assert_eq!(st, StatusCode::CREATED, "create user {username}: {json}");
}

async fn create_supplier(server: &TestServer, token: &str, code: &str, name: &str) -> i64 {
    let body = format!(r#"{{"code":"{code}","name":"{name}"}}"#);
    let (st, json) = server.req("POST", "/suppliers", body, Some(token)).await;
    assert_eq!(st, StatusCode::CREATED, "create supplier: {json}");
    json["data"]["id"].as_i64().unwrap()
}

async fn create_item(server: &TestServer, token: &str, sku: &str, name: &str) -> i64 {
    let body = format!(r#"{{"sku":"{sku}","name":"{name}"}}"#);
    let (st, json) = server.req("POST", "/items", body, Some(token)).await;
    assert_eq!(st, StatusCode::CREATED, "create item: {json}");
    json["data"]["id"].as_i64().unwrap()
}

async fn create_po(
    server: &TestServer,
    token: &str,
    supplier_id: i64,
    items: &[(i64, f64, Option<&str>)],
) -> (StatusCode, serde_json::Value) {
    let items_json: Vec<String> = items
        .iter()
        .map(|(iid, qty, price)| match price {
            Some(p) => format!(r#"{{"item_id":{iid},"quantity":{qty},"unit_price":"{p}"}}"#),
            None => format!(r#"{{"item_id":{iid},"quantity":{qty}}}"#),
        })
        .collect();
    let body = format!(
        r#"{{"supplier_id":{supplier_id},"order_date":"2026-08-10","items":[{}]}}"#,
        items_json.join(",")
    );
    server
        .req("POST", "/purchase-orders", body, Some(token))
        .await
}

#[tokio::test]
async fn create_po_with_items_returns_201() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;
    let sid = create_supplier(&server, &token, "PS-1", "供应商一").await;
    let iid_a = create_item(&server, &token, "PS-SKU-A", "钢材").await;
    let iid_b = create_item(&server, &token, "PS-SKU-B", "螺母").await;

    let (st, json) = create_po(
        &server,
        &token,
        sid,
        &[(iid_a, 10.0, Some("100.50")), (iid_b, 5.0, Some("20"))],
    )
    .await;
    assert_eq!(st, StatusCode::CREATED, "create po: {json}");
    let id = json["data"]["id"].as_i64().unwrap();
    assert!(id > 0);
    assert_eq!(json["data"]["status"], "draft");
    assert_eq!(json["data"]["doc_status"], 0);
    let order_no = json["data"]["order_no"].as_str().unwrap();
    // 生成格式: PO{YYYYMMDD}-{rand4hex}，前缀是当天 UTC 日期。基准时间硬编码会导致跨日断言失败。
    let today_prefix = format!("PO{}-", chrono::Utc::now().format("%Y%m%d"));
    assert!(
        order_no.starts_with(&today_prefix),
        "order_no {order_no} 不以前缀 {today_prefix} 开头"
    );

    let (st, json) = server
        .req(
            "GET",
            &format!("/purchase-orders/{id}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(json["data"]["order"]["id"], id);
    assert_eq!(json["data"]["order"]["order_no"], order_no);
    let items = json["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn total_amount_matches_sum_of_qty_times_price() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;
    let sid = create_supplier(&server, &token, "TA-1", "供应商TA").await;
    let iid_a = create_item(&server, &token, "TA-SKU-A", "物A").await;
    let iid_b = create_item(&server, &token, "TA-SKU-B", "物B").await;

    let (st, json) = create_po(
        &server,
        &token,
        sid,
        &[(iid_a, 10.0, Some("100.50")), (iid_b, 5.0, Some("20"))],
    )
    .await;
    assert_eq!(st, StatusCode::CREATED, "create po: {json}");
    let total = json["data"]["total_amount"].as_str().unwrap();
    let total_dec = Decimal::from_str(total).unwrap();
    let expected = Decimal::from_str("1105.00").unwrap();
    assert_eq!(total_dec, expected, "total_amount={total}");
}

#[tokio::test]
async fn list_po_filter_by_status_draft() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;
    let sid = create_supplier(&server, &token, "LF-1", "供应商LF").await;
    let iid = create_item(&server, &token, "LF-SKU", "物LF").await;
    create_po(&server, &token, sid, &[(iid, 3.0, Some("50"))]).await;
    let (_, created2) = create_po(&server, &token, sid, &[(iid, 1.0, Some("10"))]).await;
    let id2 = created2["data"]["id"].as_i64().unwrap();
    server
        .req(
            "POST",
            &format!("/purchase-orders/{id2}/submit"),
            String::new(),
            Some(&token),
        )
        .await;

    let (st, json) = server
        .req(
            "GET",
            "/purchase-orders?status=draft&page=1&page_size=20",
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    let items = json["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "draft filter: {json}");
    assert_eq!(json["meta"]["total"], 1);
    assert_eq!(items[0]["status"], "draft");
}

#[tokio::test]
async fn create_po_nonexistent_supplier_returns_404() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;
    let iid = create_item(&server, &token, "NS-SKU", "物NS").await;

    let (st, json) = create_po(&server, &token, 999999, &[(iid, 1.0, Some("10"))]).await;
    assert_eq!(st, StatusCode::NOT_FOUND, "nonexistent supplier: {json}");
    assert_eq!(json["code"], 15001);
}

#[tokio::test]
async fn create_po_nonexistent_item_returns_404() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;
    let sid = create_supplier(&server, &token, "NI-1", "供应商NI").await;

    let (st, json) = create_po(&server, &token, sid, &[(999999, 1.0, Some("10"))]).await;
    assert_eq!(st, StatusCode::NOT_FOUND, "nonexistent item: {json}");
    assert_eq!(json["code"], 12001);
}

#[tokio::test]
async fn submit_po_transitions_to_submitted() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;
    let sid = create_supplier(&server, &token, "SB-1", "供应商SB").await;
    let iid = create_item(&server, &token, "SB-SKU", "物SB").await;
    let (_, created) = create_po(&server, &token, sid, &[(iid, 2.0, Some("15"))]).await;
    let id = created["data"]["id"].as_i64().unwrap();

    let (st, json) = server
        .req(
            "POST",
            &format!("/purchase-orders/{id}/submit"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "submit: {json}");
    assert_eq!(json["data"]["status"], "submitted");
    assert_eq!(json["data"]["doc_status"], 1);
}

#[tokio::test]
async fn submit_twice_returns_cannot_modify() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;
    let sid = create_supplier(&server, &token, "S2-1", "供应商S2").await;
    let iid = create_item(&server, &token, "S2-SKU", "物S2").await;
    let (_, created) = create_po(&server, &token, sid, &[(iid, 1.0, Some("5"))]).await;
    let id = created["data"]["id"].as_i64().unwrap();
    server
        .req(
            "POST",
            &format!("/purchase-orders/{id}/submit"),
            String::new(),
            Some(&token),
        )
        .await;

    let (st, json) = server
        .req(
            "POST",
            &format!("/purchase-orders/{id}/submit"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "submit twice: {json}");
    assert_eq!(json["code"], 14001);
}

#[tokio::test]
async fn approve_draft_returns_cannot_modify() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;
    let sid = create_supplier(&server, &token, "AD-1", "供应商AD").await;
    let iid = create_item(&server, &token, "AD-SKU", "物AD").await;
    let (_, created) = create_po(&server, &token, sid, &[(iid, 1.0, Some("5"))]).await;
    let id = created["data"]["id"].as_i64().unwrap();

    let (st, json) = server
        .req(
            "POST",
            &format!("/purchase-orders/{id}/approve"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "approve draft: {json}");
    assert_eq!(json["code"], 14001);
}

#[tokio::test]
async fn approve_submitted_transitions_to_approved() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;
    let sid = create_supplier(&server, &token, "AP-1", "供应商AP").await;
    let iid = create_item(&server, &token, "AP-SKU", "物AP").await;
    let (_, created) = create_po(&server, &token, sid, &[(iid, 1.0, Some("5"))]).await;
    let id = created["data"]["id"].as_i64().unwrap();
    server
        .req(
            "POST",
            &format!("/purchase-orders/{id}/submit"),
            String::new(),
            Some(&token),
        )
        .await;

    let (st, json) = server
        .req(
            "POST",
            &format!("/purchase-orders/{id}/approve"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "approve: {json}");
    assert_eq!(json["data"]["status"], "approved");
    assert_eq!(json["data"]["doc_status"], 1);
}

#[tokio::test]
async fn reject_submitted_transitions_to_rejected() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;
    let sid = create_supplier(&server, &token, "RJ-1", "供应商RJ").await;
    let iid = create_item(&server, &token, "RJ-SKU", "物RJ").await;
    let (_, created) = create_po(&server, &token, sid, &[(iid, 1.0, Some("5"))]).await;
    let id = created["data"]["id"].as_i64().unwrap();
    server
        .req(
            "POST",
            &format!("/purchase-orders/{id}/submit"),
            String::new(),
            Some(&token),
        )
        .await;

    let (st, json) = server
        .req(
            "POST",
            &format!("/purchase-orders/{id}/reject"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "reject: {json}");
    assert_eq!(json["data"]["status"], "rejected");
    assert_eq!(json["data"]["doc_status"], 1);
}

#[tokio::test]
async fn cancel_draft_transitions_to_cancelled() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;
    let sid = create_supplier(&server, &token, "CD-1", "供应商CD").await;
    let iid = create_item(&server, &token, "CD-SKU", "物CD").await;
    let (_, created) = create_po(&server, &token, sid, &[(iid, 1.0, Some("5"))]).await;
    let id = created["data"]["id"].as_i64().unwrap();

    let (st, json) = server
        .req(
            "POST",
            &format!("/purchase-orders/{id}/cancel"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "cancel draft: {json}");
    assert_eq!(json["data"]["status"], "cancelled");
    assert_eq!(json["data"]["doc_status"], 2);
}

#[tokio::test]
async fn cancel_approved_returns_cannot_modify() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;
    let sid = create_supplier(&server, &token, "CA-1", "供应商CA").await;
    let iid = create_item(&server, &token, "CA-SKU", "物CA").await;
    let (_, created) = create_po(&server, &token, sid, &[(iid, 1.0, Some("5"))]).await;
    let id = created["data"]["id"].as_i64().unwrap();
    server
        .req(
            "POST",
            &format!("/purchase-orders/{id}/submit"),
            String::new(),
            Some(&token),
        )
        .await;
    server
        .req(
            "POST",
            &format!("/purchase-orders/{id}/approve"),
            String::new(),
            Some(&token),
        )
        .await;

    let (st, json) = server
        .req(
            "POST",
            &format!("/purchase-orders/{id}/cancel"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "cancel approved: {json}");
    assert_eq!(json["code"], 14001);
}

#[tokio::test]
async fn rbac_warehouse_read_ok_write_forbidden_purchaser_both_ok() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let admin = admin_token(&server).await;
    let sid = create_supplier(&server, &admin, "RB-1", "供应商RB").await;
    let iid = create_item(&server, &admin, "RB-SKU", "物RB").await;

    // warehouse 角色 id=3: 含 order.read, 不含 order.write
    create_user(&server, &admin, "wh3", &[3]).await;
    let wh = login_as(&server, "wh3", "pass1234").await;
    let (st, _) = server
        .req(
            "GET",
            "/purchase-orders?page=1&page_size=20",
            String::new(),
            Some(&wh),
        )
        .await;
    assert_eq!(
        st,
        StatusCode::OK,
        "warehouse GET po should be 200 (order.read)"
    );
    let (st, json) = create_po(&server, &wh, sid, &[(iid, 1.0, Some("5"))]).await;
    assert_eq!(st, StatusCode::FORBIDDEN, "warehouse POST po: {json}");
    assert_eq!(json["code"], 11003);

    // purchaser 角色 id=4: 含 order.read + order.write, 不含 order.approve
    create_user(&server, &admin, "pu4", &[4]).await;
    let pu = login_as(&server, "pu4", "pass1234").await;
    let (st, _) = server
        .req(
            "GET",
            "/purchase-orders?page=1&page_size=20",
            String::new(),
            Some(&pu),
        )
        .await;
    assert_eq!(
        st,
        StatusCode::OK,
        "purchaser GET po should be 200 (order.read)"
    );
    let (st, json) = create_po(&server, &pu, sid, &[(iid, 1.0, Some("5"))]).await;
    assert_eq!(st, StatusCode::CREATED, "purchaser POST po: {json}");
    let id = json["data"]["id"].as_i64().unwrap();

    // purchaser 无 order.approve → submit 应 403
    let (st, json) = server
        .req(
            "POST",
            &format!("/purchase-orders/{id}/submit"),
            String::new(),
            Some(&pu),
        )
        .await;
    assert_eq!(
        st,
        StatusCode::FORBIDDEN,
        "purchaser submit (no approve): {json}"
    );
    assert_eq!(json["code"], 11003);
}
