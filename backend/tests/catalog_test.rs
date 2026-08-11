//! Catalog 集成测试 — item CRUD + RBAC + 校验

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

async fn create_item(
    server: &TestServer,
    token: &str,
    sku: &str,
    name: &str,
    category: Option<&str>,
    unit: Option<&str>,
    spec: Option<&str>,
) -> (StatusCode, serde_json::Value) {
    let mut s = format!(r#"{{"sku":"{sku}","name":"{name}""#);
    if let Some(c) = category {
        s.push_str(&format!(r#","category":"{c}""#));
    }
    if let Some(u) = unit {
        s.push_str(&format!(r#","unit":"{u}""#));
    }
    if let Some(sp) = spec {
        s.push_str(&format!(r#","spec":"{sp}""#));
    }
    s.push('}');
    server.req("POST", "/items", s, Some(token)).await
}

#[tokio::test]
async fn create_and_get_item() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (st, json) = create_item(
        &server,
        &token,
        "SKU-001",
        "钢条",
        Some("原材料"),
        Some("kg"),
        Some("Φ20"),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED, "create response: {json}");
    let id = json["data"]["id"].as_i64().unwrap();
    assert!(id > 0);
    assert_eq!(json["data"]["sku"], "SKU-001");
    assert_eq!(json["data"]["name"], "钢条");
    assert_eq!(json["data"]["status"], "draft");

    let (st, json) = server
        .req("GET", &format!("/items/{id}"), String::new(), Some(&token))
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(json["data"]["sku"], "SKU-001");
    assert_eq!(json["data"]["category"], "原材料");
}

#[tokio::test]
async fn duplicate_sku_returns_400() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (st, _) = create_item(&server, &token, "DUP-1", "A", None, None, None).await;
    assert_eq!(st, StatusCode::CREATED);

    let (st, json) = create_item(&server, &token, "DUP-1", "B", None, None, None).await;
    assert_eq!(
        st,
        StatusCode::BAD_REQUEST,
        "duplicate sku response: {json}"
    );
    assert_eq!(json["code"], 12002);
    assert_eq!(json["success"], false);
}

#[tokio::test]
async fn list_with_category_filter() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    create_item(&server, &token, "C-1", "n1", Some("Foo"), None, None).await;
    create_item(&server, &token, "C-2", "n2", Some("Foo"), None, None).await;
    create_item(&server, &token, "C-3", "n3", Some("Bar"), None, None).await;

    let (st, json) = server
        .req(
            "GET",
            "/items?category=Foo&page=1&page_size=20",
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    let items = json["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 2, "Foo category items: {json}");
    assert_eq!(json["meta"]["total"], 2);
}

#[tokio::test]
async fn update_item_changes_fields() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (_, created) = create_item(&server, &token, "U-1", "Old", Some("Cat1"), None, None).await;
    let id = created["data"]["id"].as_i64().unwrap();

    let body = r#"{"sku":"U-1","name":"New","category":"Cat2","status":"active"}"#.to_string();
    let (st, json) = server
        .req("PUT", &format!("/items/{id}"), body, Some(&token))
        .await;
    assert_eq!(st, StatusCode::OK, "update response: {json}");
    assert_eq!(json["data"]["name"], "New");
    assert_eq!(json["data"]["category"], "Cat2");
    assert_eq!(json["data"]["status"], "active");
}

#[tokio::test]
async fn delete_item_then_404() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (_, created) = create_item(&server, &token, "D-1", "ToDelete", None, None, None).await;
    let id = created["data"]["id"].as_i64().unwrap();

    let (st, _) = server
        .req(
            "DELETE",
            &format!("/items/{id}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);

    let (st, json) = server
        .req("GET", &format!("/items/{id}"), String::new(), Some(&token))
        .await;
    assert_eq!(st, StatusCode::NOT_FOUND, "get after delete: {json}");
    assert_eq!(json["code"], 12001);
}

#[tokio::test]
async fn empty_sku_or_name_returns_400() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let body = r#"{"sku":"","name":"NoSku"}"#.to_string();
    let (st, json) = server.req("POST", "/items", body, Some(&token)).await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "empty sku: {json}");
    assert_eq!(json["code"], 10002);

    let body = r#"{"sku":"HAS-SKU","name":"   "}"#.to_string();
    let (st, json) = server.req("POST", "/items", body, Some(&token)).await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "blank name: {json}");
    assert_eq!(json["code"], 10002);
}

#[tokio::test]
async fn rbac_user_without_item_write_forbidden() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let admin = admin_token(&server).await;

    let body =
        r#"{"username":"mgr","password":"pass1234","display_name":"Manager","role_ids":[2]}"#
            .to_string();
    let (st, json) = server.req("POST", "/users", body, Some(&admin)).await;
    assert_eq!(st, StatusCode::CREATED, "create manager: {json}");

    let body = r#"{"username":"mgr","password":"pass1234"}"#.to_string();
    let (st, json) = server.req("POST", "/auth/login", body, None).await;
    assert_eq!(st, StatusCode::OK, "manager login: {json}");
    let mgr_token = json["data"]["access_token"].as_str().unwrap().to_string();

    let body = r#"{"sku":"RBAC-1","name":"X"}"#.to_string();
    let (st, json) = server.req("POST", "/items", body, Some(&mgr_token)).await;
    assert_eq!(st, StatusCode::FORBIDDEN, "manager post: {json}");
    assert_eq!(json["code"], 11003);

    let (st, _) = server
        .req(
            "GET",
            "/items?page=1&page_size=20",
            String::new(),
            Some(&mgr_token),
        )
        .await;
    assert_eq!(
        st,
        StatusCode::OK,
        "manager (role=manager) has item.read? check seed"
    );
}

#[tokio::test]
async fn categories_returns_distinct() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    create_item(&server, &token, "CAT-1", "n1", Some("Alpha"), None, None).await;
    create_item(&server, &token, "CAT-2", "n2", Some("Beta"), None, None).await;
    create_item(&server, &token, "CAT-3", "n3", Some("Alpha"), None, None).await;

    let (st, json) = server
        .req("GET", "/items/categories", String::new(), Some(&token))
        .await;
    assert_eq!(st, StatusCode::OK);
    let cats = json["data"].as_array().unwrap();
    assert_eq!(cats.len(), 2, "distinct categories: {json}");
}
