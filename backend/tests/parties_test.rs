//! Parties 集成测试 — supplier/customer CRUD + RBAC + 校验

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

async fn create_supplier(
    server: &TestServer,
    token: &str,
    code: &str,
    name: &str,
    contact: Option<&str>,
    phone: Option<&str>,
    address: Option<&str>,
) -> (StatusCode, serde_json::Value) {
    let mut s = format!(r#"{{"code":"{code}","name":"{name}""#);
    if let Some(c) = contact {
        s.push_str(&format!(r#","contact":"{c}""#));
    }
    if let Some(p) = phone {
        s.push_str(&format!(r#","phone":"{p}""#));
    }
    if let Some(a) = address {
        s.push_str(&format!(r#","address":"{a}""#));
    }
    s.push('}');
    server.req("POST", "/suppliers", s, Some(token)).await
}

async fn create_customer(
    server: &TestServer,
    token: &str,
    code: &str,
    name: &str,
) -> (StatusCode, serde_json::Value) {
    let s = format!(r#"{{"code":"{code}","name":"{name}"}}"#);
    server.req("POST", "/customers", s, Some(token)).await
}

#[tokio::test]
async fn create_and_get_supplier() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (st, json) = create_supplier(
        &server,
        &token,
        "SUP-001",
        "东方钢铁",
        Some("王经理"),
        Some("13800000001"),
        Some("上海市浦东新区"),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED, "create supplier: {json}");
    let id = json["data"]["id"].as_i64().unwrap();
    assert!(id > 0);
    assert_eq!(json["data"]["code"], "SUP-001");
    assert_eq!(json["data"]["name"], "东方钢铁");
    assert_eq!(json["data"]["contact"], "王经理");
    assert_eq!(json["data"]["phone"], "13800000001");
    assert_eq!(json["data"]["status"], "active");

    let (st, json) = server
        .req(
            "GET",
            &format!("/suppliers/{id}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(json["data"]["code"], "SUP-001");
    assert_eq!(json["data"]["address"], "上海市浦东新区");
}

#[tokio::test]
async fn duplicate_supplier_code_returns_400() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (st, _) = create_supplier(&server, &token, "DUP-S1", "A", None, None, None).await;
    assert_eq!(st, StatusCode::CREATED);

    let (st, json) = create_supplier(&server, &token, "DUP-S1", "B", None, None, None).await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "duplicate supplier: {json}");
    assert_eq!(json["code"], 10002);
    assert_eq!(json["success"], false);
}

#[tokio::test]
async fn list_suppliers_with_filter() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    create_supplier(&server, &token, "L-1", "南京钢铁", None, None, None).await;
    create_supplier(&server, &token, "L-2", "南京物流", None, None, None).await;
    create_supplier(&server, &token, "L-3", "北京物资", None, None, None).await;

    let (st, json) = server
        .req(
            "GET",
            "/suppliers?name=%E5%8D%97%E4%BA%AC&page=1&page_size=20",
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    let items = json["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 2, "name=南京 filter: {json}");
    assert_eq!(json["meta"]["total"], 2);
}

#[tokio::test]
async fn update_supplier_changes_fields() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (_, created) = create_supplier(&server, &token, "U-S1", "旧名称", None, None, None).await;
    let id = created["data"]["id"].as_i64().unwrap();

    let body = r#"{"code":"U-S1","name":"新名称","contact":"李四","phone":"13900000000","status":"inactive"}"#.to_string();
    let (st, json) = server
        .req("PUT", &format!("/suppliers/{id}"), body, Some(&token))
        .await;
    assert_eq!(st, StatusCode::OK, "update supplier: {json}");
    assert_eq!(json["data"]["name"], "新名称");
    assert_eq!(json["data"]["contact"], "李四");
    assert_eq!(json["data"]["phone"], "13900000000");
    assert_eq!(json["data"]["status"], "inactive");
}

#[tokio::test]
async fn delete_supplier_then_404() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (_, created) = create_supplier(&server, &token, "D-S1", "待删除", None, None, None).await;
    let id = created["data"]["id"].as_i64().unwrap();

    let (st, _) = server
        .req(
            "DELETE",
            &format!("/suppliers/{id}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);

    let (st, json) = server
        .req(
            "GET",
            &format!("/suppliers/{id}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::NOT_FOUND, "get after delete: {json}");
    assert_eq!(json["code"], 15001);
}

#[tokio::test]
async fn customer_create_and_list() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (st, json) = create_customer(&server, &token, "CUS-001", "甲方公司").await;
    assert_eq!(st, StatusCode::CREATED, "create customer: {json}");
    let id = json["data"]["id"].as_i64().unwrap();
    assert!(id > 0);
    assert_eq!(json["data"]["code"], "CUS-001");
    assert_eq!(json["data"]["status"], "active");

    create_customer(&server, &token, "CUS-002", "乙方公司").await;

    let (st, json) = server
        .req(
            "GET",
            "/customers?page=1&page_size=20",
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    let items = json["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 2, "customer list: {json}");
    assert_eq!(json["meta"]["total"], 2);
}

#[tokio::test]
async fn customer_update_and_delete() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (_, created) = create_customer(&server, &token, "CU-1", "原客户").await;
    let id = created["data"]["id"].as_i64().unwrap();

    let body = r#"{"code":"CU-1","name":"更名客户","status":"inactive"}"#.to_string();
    let (st, json) = server
        .req("PUT", &format!("/customers/{id}"), body, Some(&token))
        .await;
    assert_eq!(st, StatusCode::OK, "update customer: {json}");
    assert_eq!(json["data"]["name"], "更名客户");
    assert_eq!(json["data"]["status"], "inactive");

    let (st, _) = server
        .req(
            "DELETE",
            &format!("/customers/{id}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);

    let (st, json) = server
        .req(
            "GET",
            &format!("/customers/{id}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::NOT_FOUND, "customer after delete: {json}");
    assert_eq!(json["code"], 15002);
}

#[tokio::test]
async fn empty_code_or_name_returns_400() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let body = r#"{"code":"","name":"NoCode"}"#.to_string();
    let (st, json) = server.req("POST", "/suppliers", body, Some(&token)).await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "empty code: {json}");
    assert_eq!(json["code"], 10002);

    let body = r#"{"code":"HAS-CODE","name":"   "}"#.to_string();
    let (st, json) = server.req("POST", "/customers", body, Some(&token)).await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "blank name: {json}");
    assert_eq!(json["code"], 10002);
}

#[tokio::test]
async fn rbac_manager_read_ok_write_forbidden() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let admin = admin_token(&server).await;

    // manager 角色为 role id=2，种子权限含 order.read，不含 order.write
    let body =
        r#"{"username":"mgr","password":"pass1234","display_name":"Manager","role_ids":[2]}"#
            .to_string();
    let (st, json) = server.req("POST", "/users", body, Some(&admin)).await;
    assert_eq!(st, StatusCode::CREATED, "create manager: {json}");

    let body = r#"{"username":"mgr","password":"pass1234"}"#.to_string();
    let (st, json) = server.req("POST", "/auth/login", body, None).await;
    assert_eq!(st, StatusCode::OK, "manager login: {json}");
    let mgr_token = json["data"]["access_token"].as_str().unwrap().to_string();

    // GET（order.read） → 200
    let (st, _) = server
        .req(
            "GET",
            "/suppliers?page=1&page_size=20",
            String::new(),
            Some(&mgr_token),
        )
        .await;
    assert_eq!(
        st,
        StatusCode::OK,
        "manager GET supplier should be 200 (order.read)"
    );

    // POST（order.write） → 403
    let body = r#"{"code":"RBAC-S1","name":"X"}"#.to_string();
    let (st, json) = server
        .req("POST", "/suppliers", body, Some(&mgr_token))
        .await;
    assert_eq!(st, StatusCode::FORBIDDEN, "manager POST supplier: {json}");
    assert_eq!(json["code"], 11003);

    // customer 同理
    let (st, _) = server
        .req(
            "GET",
            "/customers?page=1&page_size=20",
            String::new(),
            Some(&mgr_token),
        )
        .await;
    assert_eq!(
        st,
        StatusCode::OK,
        "manager GET customer should be 200 (order.read)"
    );

    let body = r#"{"code":"RBAC-C1","name":"Y"}"#.to_string();
    let (st, json) = server
        .req("POST", "/customers", body, Some(&mgr_token))
        .await;
    assert_eq!(st, StatusCode::FORBIDDEN, "manager POST customer: {json}");
    assert_eq!(json["code"], 11003);
}
