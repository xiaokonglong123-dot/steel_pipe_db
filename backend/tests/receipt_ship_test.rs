mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use sqlx::SqlitePool;
use tempfile::TempDir;
use tower::ServiceExt;

use common::test_pool;
use erp_v2::auth::bootstrap_admin;
use erp_v2::config::Config;
use erp_v2::http::router;

struct TestServer {
    app: Router,
    _dir: TempDir,
}

impl TestServer {
    async fn new(pool: SqlitePool, dir: TempDir) -> Self {
        let _cfg = Config {
            jwt_expiry_hours: 1,
            refresh_expiry_days: 1,
            ..Config::from_env().expect("config")
        };
        Self {
            app: router(pool, "test-secret".to_string()),
            _dir: dir,
        }
    }

    async fn req(
        &self,
        method: &str,
        path: &str,
        body: &str,
        token: Option<&str>,
    ) -> (StatusCode, serde_json::Value) {
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(token) = token {
            builder = builder.header("Authorization", format!("Bearer {token}"));
        }
        let request = builder
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .expect("request");
        let response = self.app.clone().oneshot(request).await.expect("response");
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("body");
        (
            status,
            serde_json::from_slice(&bytes).expect("json response"),
        )
    }
}

async fn fixture() -> (TestServer, String, i64, i64, i64, i64) {
    let (pool, dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool, dir).await;
    let (_, login) = server
        .req(
            "POST",
            "/auth/login",
            r#"{"username":"admin","password":"admin123"}"#,
            None,
        )
        .await;
    let token = login["data"]["access_token"].as_str().unwrap().to_owned();
    let (supplier_status, supplier) = server
        .req(
            "POST",
            "/suppliers",
            r#"{"code":"R-SUP","name":"收货供应商"}"#,
            Some(&token),
        )
        .await;
    assert_eq!(supplier_status, StatusCode::CREATED, "supplier: {supplier}");
    let (item_status, item) = server
        .req(
            "POST",
            "/items",
            r#"{"sku":"R-SKU","name":"收货商品"}"#,
            Some(&token),
        )
        .await;
    assert_eq!(item_status, StatusCode::CREATED, "item: {item}");
    let (location_status, location) = server
        .req(
            "POST",
            "/locations",
            r#"{"code":"R-LOC","name":"收货库位"}"#,
            Some(&token),
        )
        .await;
    assert_eq!(location_status, StatusCode::CREATED, "location: {location}");
    let (customer_status, customer) = server
        .req(
            "POST",
            "/customers",
            r#"{"code":"R-CUST","name":"发货客户"}"#,
            Some(&token),
        )
        .await;
    assert_eq!(customer_status, StatusCode::CREATED, "customer: {customer}");
    (
        server,
        token,
        supplier["data"]["id"].as_i64().unwrap(),
        item["data"]["id"].as_i64().unwrap(),
        location["data"]["id"].as_i64().unwrap(),
        customer["data"]["id"].as_i64().unwrap(),
    )
}

async fn create_po(server: &TestServer, token: &str, supplier: i64, item: i64, qty: f64) -> i64 {
    let body = format!(
        r#"{{"supplier_id":{supplier},"order_date":"2026-08-10","items":[{{"item_id":{item},"quantity":{qty},"unit_price":"1"}}]}}"#
    );
    let (status, json) = server
        .req("POST", "/purchase-orders", &body, Some(token))
        .await;
    assert_eq!(status, StatusCode::CREATED, "create po: {json}");
    json["data"]["id"].as_i64().unwrap()
}

async fn create_so(server: &TestServer, token: &str, customer: i64, item: i64, qty: f64) -> i64 {
    let body = format!(
        r#"{{"customer_id":{customer},"items":[{{"item_id":{item},"quantity":{qty},"unit_price":"1"}}]}}"#
    );
    let (status, json) = server
        .req("POST", "/sales-orders", &body, Some(token))
        .await;
    assert_eq!(status, StatusCode::CREATED, "create so: {json}");
    json["data"]["id"].as_i64().unwrap()
}

async fn post_stock(server: &TestServer, token: &str, item: i64, location: i64, qty: f64) {
    let body = format!(
        r#"{{"inbound_type":"purchase","items":[{{"item_id":{item},"location_id":{location},"quantity":{qty}}}]}}"#
    );
    let (status, json) = server.req("POST", "/inbounds", &body, Some(token)).await;
    assert_eq!(status, StatusCode::CREATED, "create inbound: {json}");
    let id = json["data"]["id"].as_i64().unwrap();
    let (status, _) = server
        .req("POST", &format!("/inbounds/{id}/post"), "", Some(token))
        .await;
    assert_eq!(status, StatusCode::OK);
}

async fn approve(server: &TestServer, token: &str, path: &str) {
    let (status, json) = server.req("POST", path, "", Some(token)).await;
    assert_eq!(status, StatusCode::OK, "{path}: {json}");
}

#[tokio::test]
async fn approved_po_receive_increases_balance() {
    let (server, token, supplier, item, location, _) = fixture().await;
    let po = create_po(&server, &token, supplier, item, 4.0).await;
    approve(&server, &token, &format!("/purchase-orders/{po}/submit")).await;
    approve(&server, &token, &format!("/purchase-orders/{po}/approve")).await;
    let body =
        format!(r#"{{"items":[{{"item_id":{item},"location_id":{location},"quantity":4}}]}}"#);
    let (status, _) = server
        .req(
            "POST",
            &format!("/purchase-orders/{po}/receive"),
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let (_, stock) = server
        .req(
            "GET",
            &format!("/stock?item_id={item}&location_id={location}"),
            "",
            Some(&token),
        )
        .await;
    assert_eq!(stock["data"]["items"][0]["quantity"], 4.0);
}

#[tokio::test]
async fn receive_requires_approved_po() {
    let (server, token, supplier, item, location, _) = fixture().await;
    let po = create_po(&server, &token, supplier, item, 1.0).await;
    let body =
        format!(r#"{{"items":[{{"item_id":{item},"location_id":{location},"quantity":1}}]}}"#);
    let (status, json) = server
        .req(
            "POST",
            &format!("/purchase-orders/{po}/receive"),
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["code"], 14001);
}

#[tokio::test]
async fn receive_unknown_po_returns_404() {
    let (server, token, _, item, location, _) = fixture().await;
    let body =
        format!(r#"{{"items":[{{"item_id":{item},"location_id":{location},"quantity":1}}]}}"#);
    let (status, json) = server
        .req(
            "POST",
            "/purchase-orders/999999/receive",
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json["code"], 14002);
}

#[tokio::test]
async fn receive_unknown_item_returns_12001() {
    let (server, token, supplier, item, location, _) = fixture().await;
    let po = create_po(&server, &token, supplier, item, 1.0).await;
    approve(&server, &token, &format!("/purchase-orders/{po}/submit")).await;
    approve(&server, &token, &format!("/purchase-orders/{po}/approve")).await;
    let body =
        format!(r#"{{"items":[{{"item_id":999999,"location_id":{location},"quantity":1}}]}}"#);
    let (status, json) = server
        .req(
            "POST",
            &format!("/purchase-orders/{po}/receive"),
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json["code"], 12001);
}

#[tokio::test]
async fn approved_so_ship_decreases_balance_and_releases_reservation() {
    let (server, token, _, item, location, customer) = fixture().await;
    post_stock(&server, &token, item, location, 5.0).await;
    let so = create_so(&server, &token, customer, item, 3.0).await;
    approve(&server, &token, &format!("/sales-orders/{so}/submit")).await;
    approve(&server, &token, &format!("/sales-orders/{so}/approve")).await;
    let body =
        format!(r#"{{"items":[{{"item_id":{item},"location_id":{location},"quantity":3}}]}}"#);
    let (status, _) = server
        .req(
            "POST",
            &format!("/sales-orders/{so}/ship"),
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let (_, stock) = server
        .req(
            "GET",
            &format!("/stock?item_id={item}&location_id={location}"),
            "",
            Some(&token),
        )
        .await;
    assert_eq!(stock["data"]["items"][0]["quantity"], 2.0);
    let (_, reservations) = server
        .req(
            "GET",
            &format!("/reservations?item_id={item}"),
            "",
            Some(&token),
        )
        .await;
    assert!(reservations["data"]["items"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn ship_requires_approved_so() {
    let (server, token, _, item, location, customer) = fixture().await;
    let so = create_so(&server, &token, customer, item, 1.0).await;
    let body =
        format!(r#"{{"items":[{{"item_id":{item},"location_id":{location},"quantity":1}}]}}"#);
    let (status, json) = server
        .req(
            "POST",
            &format!("/sales-orders/{so}/ship"),
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["code"], 14001);
}

#[tokio::test]
async fn ship_returns_insufficient_stock() {
    let (server, token, _, item, location, customer) = fixture().await;
    post_stock(&server, &token, item, location, 1.0).await;
    let so = create_so(&server, &token, customer, item, 1.0).await;
    approve(&server, &token, &format!("/sales-orders/{so}/submit")).await;
    approve(&server, &token, &format!("/sales-orders/{so}/approve")).await;
    let body =
        format!(r#"{{"items":[{{"item_id":{item},"location_id":{location},"quantity":2}}]}}"#);
    let (status, json) = server
        .req(
            "POST",
            &format!("/sales-orders/{so}/ship"),
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["code"], 13001);
}

#[tokio::test]
async fn submit_creates_completed_workflow_after_approval() {
    let (server, token, supplier, item, _, _) = fixture().await;
    let po = create_po(&server, &token, supplier, item, 1.0).await;
    approve(&server, &token, &format!("/purchase-orders/{po}/submit")).await;
    let (_, pending) = server
        .req(
            "GET",
            "/workflow-instances?business_type=purchase_order",
            "",
            Some(&token),
        )
        .await;
    assert_eq!(pending["data"]["items"][0]["status"], "active");
    approve(&server, &token, &format!("/purchase-orders/{po}/approve")).await;
    let (_, completed) = server
        .req(
            "GET",
            "/workflow-instances?business_type=purchase_order",
            "",
            Some(&token),
        )
        .await;
    assert_eq!(completed["data"]["items"][0]["status"], "completed");
}

#[tokio::test]
async fn ship_requires_stock_write_and_allows_warehouse_role() {
    let (server, admin, _, item, location, customer) = fixture().await;
    let manager_body =
        r#"{"username":"manager-r","password":"pass1234","display_name":"经理","role_ids":[2]}"#;
    let warehouse_body =
        r#"{"username":"warehouse-r","password":"pass1234","display_name":"仓库","role_ids":[3]}"#;
    let (status, _) = server
        .req("POST", "/users", manager_body, Some(&admin))
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = server
        .req("POST", "/users", warehouse_body, Some(&admin))
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let (_, manager_login) = server
        .req(
            "POST",
            "/auth/login",
            r#"{"username":"manager-r","password":"pass1234"}"#,
            None,
        )
        .await;
    let (_, warehouse_login) = server
        .req(
            "POST",
            "/auth/login",
            r#"{"username":"warehouse-r","password":"pass1234"}"#,
            None,
        )
        .await;
    let manager = manager_login["data"]["access_token"].as_str().unwrap();
    let warehouse = warehouse_login["data"]["access_token"].as_str().unwrap();
    post_stock(&server, &admin, item, location, 1.0).await;
    let so = create_so(&server, &admin, customer, item, 1.0).await;
    approve(&server, &admin, &format!("/sales-orders/{so}/submit")).await;
    approve(&server, &admin, &format!("/sales-orders/{so}/approve")).await;
    let body =
        format!(r#"{{"items":[{{"item_id":{item},"location_id":{location},"quantity":1}}]}}"#);
    let (status, json) = server
        .req(
            "POST",
            &format!("/sales-orders/{so}/ship"),
            &body,
            Some(manager),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(json["code"], 11003);
    let (status, _) = server
        .req(
            "POST",
            &format!("/sales-orders/{so}/ship"),
            &body,
            Some(warehouse),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
}
