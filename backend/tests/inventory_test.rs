//! Inventory 集成测试 — 入库/出库/库存/日志 + RBAC + 事务一致性

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

async fn create_inbound(
    server: &TestServer,
    token: &str,
    inbound_type: &str,
    items: &[(i64, i64, f64)],
) -> (StatusCode, serde_json::Value) {
    let items_json: Vec<String> = items
        .iter()
        .map(|(i, l, q)| format!(r#"{{"item_id":{i},"location_id":{l},"quantity":{q}}}"#))
        .collect();
    let body = format!(
        r#"{{"inbound_type":"{inbound_type}","items":[{}]}}"#,
        items_json.join(",")
    );
    server.req("POST", "/inbounds", body, Some(token)).await
}

async fn create_outbound(
    server: &TestServer,
    token: &str,
    outbound_type: &str,
    items: &[(i64, i64, f64)],
) -> (StatusCode, serde_json::Value) {
    let items_json: Vec<String> = items
        .iter()
        .map(|(i, l, q)| format!(r#"{{"item_id":{i},"location_id":{l},"quantity":{q}}}"#))
        .collect();
    let body = format!(
        r#"{{"outbound_type":"{outbound_type}","items":[{}]}}"#,
        items_json.join(",")
    );
    server.req("POST", "/outbounds", body, Some(token)).await
}

async fn post_inbound(
    server: &TestServer,
    token: &str,
    id: i64,
) -> (StatusCode, serde_json::Value) {
    server
        .req(
            "POST",
            &format!("/inbounds/{id}/post"),
            String::new(),
            Some(token),
        )
        .await
}

async fn post_outbound(
    server: &TestServer,
    token: &str,
    id: i64,
) -> (StatusCode, serde_json::Value) {
    server
        .req(
            "POST",
            &format!("/outbounds/{id}/post"),
            String::new(),
            Some(token),
        )
        .await
}

async fn stock_for(
    server: &TestServer,
    token: &str,
    item_id: i64,
    location_id: i64,
) -> Option<f64> {
    let (st, json) = server
        .req(
            "GET",
            &format!("/stock?item_id={item_id}&location_id={location_id}&page=1&page_size=200"),
            String::new(),
            Some(token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "stock query: {json}");
    let items = json["data"]["items"].as_array().unwrap();
    items
        .iter()
        .find(|r| r["item_id"] == item_id && r["location_id"] == location_id)
        .and_then(|r| r["quantity"].as_f64())
}

// —— Tests ——

#[tokio::test]
async fn create_and_post_inbound_updates_stock() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item_id = create_item(&server, &token, "INV-1").await;
    let loc_id = create_location(&server, &token, "LOC-1").await;

    let (st, json) = create_inbound(&server, &token, "purchase", &[(item_id, loc_id, 10.0)]).await;
    assert_eq!(st, StatusCode::CREATED, "create inbound: {json}");
    let inbound_id = json["data"]["id"].as_i64().unwrap();
    assert_eq!(json["data"]["status"], "draft");

    // stock should be empty before posting
    assert!(stock_for(&server, &token, item_id, loc_id).await.is_none());

    let (st, json) = post_inbound(&server, &token, inbound_id).await;
    assert_eq!(st, StatusCode::OK, "post inbound: {json}");
    assert_eq!(json["data"]["status"], "posted");

    let balance = stock_for(&server, &token, item_id, loc_id).await;
    assert_eq!(balance, Some(10.0), "stock after inbound: {balance:?}");
}

#[tokio::test]
async fn post_inbound_writes_inventory_log() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item_id = create_item(&server, &token, "LOG-1").await;
    let loc_id = create_location(&server, &token, "LOGLOC-1").await;

    let (st, json) = create_inbound(&server, &token, "purchase", &[(item_id, loc_id, 25.0)]).await;
    let inbound_id = json["data"]["id"].as_i64().unwrap();
    assert_eq!(st, StatusCode::CREATED);

    post_inbound(&server, &token, inbound_id).await;

    let (st, json) = server
        .req(
            "GET",
            &format!("/inventory-logs?item_id={item_id}&page=1&page_size=20"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "logs: {json}");
    let logs = json["data"]["items"].as_array().unwrap();
    assert_eq!(logs.len(), 1, "one log row: {json}");
    assert_eq!(logs[0]["change_type"], "inbound");
    assert_eq!(logs[0]["quantity"], 25.0);
    assert_eq!(logs[0]["ref_type"], "inbound");
    assert_eq!(logs[0]["ref_id"], inbound_id);
    // balance_after is encoded in notes
    let notes = logs[0]["notes"].as_str().unwrap();
    assert!(notes.contains("balance_after=25"), "notes: {notes}");
}

#[tokio::test]
async fn post_inbound_twice_returns_conflict() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item_id = create_item(&server, &token, "TWICE-1").await;
    let loc_id = create_location(&server, &token, "TWICELOC-1").await;

    let (_, json) = create_inbound(&server, &token, "purchase", &[(item_id, loc_id, 5.0)]).await;
    let inbound_id = json["data"]["id"].as_i64().unwrap();

    let (st, _) = post_inbound(&server, &token, inbound_id).await;
    assert_eq!(st, StatusCode::OK);

    let (st, json) = post_inbound(&server, &token, inbound_id).await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "second post: {json}");
    // OrderCannotModify = 14001
    assert_eq!(json["code"], 14001);
}

#[tokio::test]
async fn post_outbound_without_stock_returns_insufficient() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item_id = create_item(&server, &token, "OUTNO-1").await;
    let loc_id = create_location(&server, &token, "OUTNOLOC-1").await;

    let (_, json) = create_outbound(&server, &token, "other", &[(item_id, loc_id, 7.0)]).await;
    let outbound_id = json["data"]["id"].as_i64().unwrap();

    let (st, json) = post_outbound(&server, &token, outbound_id).await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "outbound no stock: {json}");
    // InsufficientStock = 13001
    assert_eq!(json["code"], 13001);
    assert_eq!(json["success"], false);
}

#[tokio::test]
async fn inbound_then_partial_outbound_leaves_remaining() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item_id = create_item(&server, &token, "PART-1").await;
    let loc_id = create_location(&server, &token, "PARTLOC-1").await;

    let (_, json) = create_inbound(&server, &token, "purchase", &[(item_id, loc_id, 10.0)]).await;
    let inbound_id = json["data"]["id"].as_i64().unwrap();
    post_inbound(&server, &token, inbound_id).await;

    let (_, json) = create_outbound(&server, &token, "other", &[(item_id, loc_id, 8.0)]).await;
    let outbound_id = json["data"]["id"].as_i64().unwrap();
    let (st, _) = post_outbound(&server, &token, outbound_id).await;
    assert_eq!(st, StatusCode::OK, "post outbound partial");

    let balance = stock_for(&server, &token, item_id, loc_id).await;
    assert_eq!(
        balance,
        Some(2.0),
        "remaining stock after partial out: {balance:?}"
    );
}

#[tokio::test]
async fn outbound_exceeding_stock_fails() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item_id = create_item(&server, &token, "EXC-1").await;
    let loc_id = create_location(&server, &token, "EXCLOC-1").await;

    let (_, json) = create_inbound(&server, &token, "purchase", &[(item_id, loc_id, 3.0)]).await;
    let inbound_id = json["data"]["id"].as_i64().unwrap();
    post_inbound(&server, &token, inbound_id).await;

    let (_, json) = create_outbound(&server, &token, "other", &[(item_id, loc_id, 5.0)]).await;
    let outbound_id = json["data"]["id"].as_i64().unwrap();
    let (st, json) = post_outbound(&server, &token, outbound_id).await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "outbound exceeds: {json}");
    assert_eq!(json["code"], 13001);

    // stock remains untouched (transaction rolled back)
    let balance = stock_for(&server, &token, item_id, loc_id).await;
    assert_eq!(
        balance,
        Some(3.0),
        "stock intact after failed outbound: {balance:?}"
    );

    // order remains in draft status
    let (st, json) = server
        .req(
            "GET",
            &format!("/outbounds/{outbound_id}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(
        json["data"]["order"]["status"], "draft",
        "order stays draft on failure: {json}"
    );
}

#[tokio::test]
async fn list_stock_filtered_by_item() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item_a = create_item(&server, &token, "STKA").await;
    let item_b = create_item(&server, &token, "STKB").await;
    let loc = create_location(&server, &token, "STKLOC").await;

    let (_, j1) = create_inbound(&server, &token, "purchase", &[(item_a, loc, 4.0)]).await;
    post_inbound(&server, &token, j1["data"]["id"].as_i64().unwrap()).await;
    let (_, j2) = create_inbound(&server, &token, "purchase", &[(item_b, loc, 9.0)]).await;
    post_inbound(&server, &token, j2["data"]["id"].as_i64().unwrap()).await;

    let (st, json) = server
        .req(
            "GET",
            &format!("/stock?item_id={item_a}&page=1&page_size=20"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    let items = json["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "stock filtered by item_a: {json}");
    assert_eq!(items[0]["item_id"], item_a);
    assert_eq!(items[0]["quantity"], 4.0);
    assert_eq!(json["meta"]["total"], 1);
}

#[tokio::test]
async fn list_logs_filtered_by_item() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item_a = create_item(&server, &token, "LGA").await;
    let item_b = create_item(&server, &token, "LGB").await;
    let loc = create_location(&server, &token, "LGLOC").await;

    let (_, j1) = create_inbound(&server, &token, "purchase", &[(item_a, loc, 1.0)]).await;
    post_inbound(&server, &token, j1["data"]["id"].as_i64().unwrap()).await;
    let (_, j2) = create_inbound(&server, &token, "purchase", &[(item_b, loc, 1.0)]).await;
    post_inbound(&server, &token, j2["data"]["id"].as_i64().unwrap()).await;

    let (st, json) = server
        .req(
            "GET",
            &format!("/inventory-logs?item_id={item_a}&page=1&page_size=20"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    let logs = json["data"]["items"].as_array().unwrap();
    assert_eq!(logs.len(), 1, "logs filtered by item_a: {json}");
    assert_eq!(logs[0]["item_id"], item_a);
}

#[tokio::test]
async fn create_inbound_with_nonexistent_item_returns_404() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let loc_id = create_location(&server, &token, "NEILOC").await;
    let (st, json) = create_inbound(&server, &token, "purchase", &[(99999, loc_id, 1.0)]).await;
    assert_eq!(st, StatusCode::NOT_FOUND, "nonexistent item: {json}");
    // ItemNotFound = 12001
    assert_eq!(json["code"], 12001);
}

#[tokio::test]
async fn create_inbound_with_nonexistent_location_returns_404() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let item_id = create_item(&server, &token, "NEL").await;
    let (st, json) = create_inbound(&server, &token, "purchase", &[(item_id, 99999, 1.0)]).await;
    assert_eq!(st, StatusCode::NOT_FOUND, "nonexistent location: {json}");
    // LocationNotFound = 13002
    assert_eq!(json["code"], 13002);
}

#[tokio::test]
async fn rbac_finance_role_forbidden_warehouse_role_allowed() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let admin = admin_token(&server).await;

    // finance role = id=6 (no stock.read, no stock.write per seed)
    let body =
        r#"{"username":"fin","password":"pass1234","display_name":"Finance","role_ids":[6]}"#
            .to_string();
    let (st, json) = server.req("POST", "/users", body, Some(&admin)).await;
    assert_eq!(st, StatusCode::CREATED, "create finance user: {json}");
    let fin_token = login_as(&server, "fin", "pass1234").await;

    // warehouse role = id=3 (has stock.read + stock.write)
    let body =
        r#"{"username":"wh","password":"pass1234","display_name":"Warehouse","role_ids":[3]}"#
            .to_string();
    let (st, json) = server.req("POST", "/users", body, Some(&admin)).await;
    assert_eq!(st, StatusCode::CREATED, "create warehouse user: {json}");
    let wh_token = login_as(&server, "wh", "pass1234").await;

    // finance GET /inbounds -> 403
    let (st, json) = server
        .req(
            "GET",
            "/inbounds?page=1&page_size=20",
            String::new(),
            Some(&fin_token),
        )
        .await;
    assert_eq!(st, StatusCode::FORBIDDEN, "finance GET inbounds: {json}");
    assert_eq!(json["code"], 11003);

    // warehouse GET /inbounds -> 200
    let (st, _) = server
        .req(
            "GET",
            "/inbounds?page=1&page_size=20",
            String::new(),
            Some(&wh_token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "warehouse GET inbounds should be 200");
}

#[tokio::test]
async fn post_inbound_requires_stock_write_not_read() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let admin = admin_token(&server).await;

    // manager role = id=2 (has stock.read, NOT stock.write per seed)
    let body =
        r#"{"username":"mgr","password":"pass1234","display_name":"Manager","role_ids":[2]}"#
            .to_string();
    let (st, json) = server.req("POST", "/users", body, Some(&admin)).await;
    assert_eq!(st, StatusCode::CREATED, "create manager: {json}");
    let mgr_token = login_as(&server, "mgr", "pass1234").await;

    // As admin, create an inbound order first
    let item_id = create_item(&server, &admin, "PERM-1").await;
    let loc_id = create_location(&server, &admin, "PERMLOC-1").await;
    let (_, j) = create_inbound(&server, &admin, "purchase", &[(item_id, loc_id, 2.0)]).await;
    let inbound_id = j["data"]["id"].as_i64().unwrap();

    // manager (stock.read, no stock.write) attempting POST /inbounds/{id}/post -> 403
    let (st, json) = post_inbound(&server, &mgr_token, inbound_id).await;
    assert_eq!(st, StatusCode::FORBIDDEN, "manager post inbound: {json}");
    assert_eq!(json["code"], 11003);

    // sanity: the order is still draft (not posted) because the request was rejected at middleware
    let (st, json) = server
        .req(
            "GET",
            &format!("/inbounds/{inbound_id}"),
            String::new(),
            Some(&admin),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    let status = json["data"]["order"]["status"].as_str().unwrap_or("");
    assert_eq!(
        status, "draft",
        "order still draft after forbidden post: {json}"
    );
}

#[tokio::test]
async fn available_qty_reflects_reservations() {
    use erp_v2::services::{inventory_service, sales_service, purchase_service, receipt_service, catalog_service, parties_service, location_service};
    use erp_v2::services::purchase_service::{CreatePurchaseOrderRequest, PurchaseOrderItemInput};
    use erp_v2::services::sales_service::{CreateSalesOrderRequest, CreateSalesOrderItemInput};
    use erp_v2::services::receipt_service::ReceivedItemInput;
    use erp_v2::auth::bootstrap_admin;

    let (pool, _dir) = common::test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let user = erp_v2_test_user();
    let supplier = parties_service::create_supplier(&pool, "S002", "S", None, None, None, None).await.unwrap();
    let customer = parties_service::create_customer(&pool, "C002", "C", None, None, None, None).await.unwrap();
    let item = catalog_service::create_item(&pool, "ATP-001", "ATP test", Some("test"), Some("个"), None).await.unwrap();
    let wh = location_service::create_warehouse(&pool, "W02", "W", None).await.unwrap();
    let loc = location_service::create_location(&pool, Some(wh.id), "W02-A1", "A").await.unwrap();

    // 收货 100
    let po = purchase_service::create_order(&pool, &CreatePurchaseOrderRequest {
        supplier_id: supplier.id, order_date: "2026-08-10".into(),
        currency: None, notes: None,
        items: vec![PurchaseOrderItemInput { item_id: item.id, quantity: 100.0, unit_price: Some("10.00".into()), notes: None }],
    }, &user).await.unwrap();
    purchase_service::submit(&pool, po.id, &user).await.unwrap();
    purchase_service::approve(&pool, po.id, &user).await.unwrap();
    receipt_service::receive_purchase_order(&pool, po.id, &[
        ReceivedItemInput { item_id: item.id, location_id: loc.id, quantity: 100.0 },
    ], &user).await.unwrap();

    // 库存 100 → available = 100 - 0 = 100
    let avail0 = inventory_service::get_available_qty(&pool, item.id, Some(loc.id)).await.unwrap();
    assert!((avail0 - 100.0).abs() < 0.01, "无预留时可用量应为 100, 实际 {avail0}");

    // 创建 SO 销 40 + submit → 自动预留 40
    let so = sales_service::create_order(&pool, &CreateSalesOrderRequest {
        customer_id: customer.id, order_date: Some("2026-08-10".into()),
        currency: None, notes: None,
        items: vec![CreateSalesOrderItemInput { item_id: item.id, quantity: 40.0, unit_price: "20.00".into(), notes: None }],
    }, &user).await.unwrap();
    sales_service::submit(&pool, so.id, &user).await.unwrap();

    // 库存仍 100，预留 40，available = 60
    let avail1 = inventory_service::get_available_qty(&pool, item.id, Some(loc.id)).await.unwrap();
    assert!((avail1 - 60.0).abs() < 0.01, "预留 40 后可用量应为 60, 实际 {avail1}");

    // 不带 location 的查询 = 余额合计 - 所有预留 = 100 - 40 = 60
    let avail2 = inventory_service::get_available_qty(&pool, item.id, None).await.unwrap();
    assert!((avail2 - 60.0).abs() < 0.01, "跨库位查询可用量应为 60, 实际 {avail2}");
}

fn erp_v2_test_user() -> erp_v2::middleware::auth::AuthUser {
    erp_v2::middleware::auth::AuthUser {
        id: 1, username: "admin".into(), display_name: "Administrator".into(),
        permissions: vec!["item.read".into(), "item.write".into(), "stock.read".into(), "stock.write".into(),
            "order.read".into(), "order.write".into(), "order.approve".into(),
            "finance.read".into(), "finance.write".into(), "report.read".into(), "user.manage".into()],
    }
}
