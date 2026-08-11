//! Locations 集成测试 — warehouse/location CRUD + RBAC + 校验

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

async fn create_warehouse(
    server: &TestServer,
    token: &str,
    code: &str,
    name: &str,
    address: Option<&str>,
) -> (StatusCode, serde_json::Value) {
    let mut s = format!(r#"{{"code":"{code}","name":"{name}""#);
    if let Some(a) = address {
        s.push_str(&format!(r#","address":"{a}""#));
    }
    s.push('}');
    server.req("POST", "/warehouses", s, Some(token)).await
}

async fn create_location(
    server: &TestServer,
    token: &str,
    warehouse_id: Option<i64>,
    code: &str,
    name: &str,
) -> (StatusCode, serde_json::Value) {
    let wid = match warehouse_id {
        Some(id) => format!(r#","warehouse_id":{id}"#),
        None => String::new(),
    };
    let body = format!(r#"{{"code":"{code}","name":"{name}"{wid}}}"#);
    server.req("POST", "/locations", body, Some(token)).await
}

#[tokio::test]
async fn create_and_get_warehouse() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (st, json) = create_warehouse(&server, &token, "WH-01", "主仓", Some("上海市")).await;
    assert_eq!(st, StatusCode::CREATED, "create warehouse: {json}");
    let id = json["data"]["id"].as_i64().unwrap();
    assert!(id > 0);
    assert_eq!(json["data"]["code"], "WH-01");
    assert_eq!(json["data"]["name"], "主仓");
    assert_eq!(json["data"]["address"], "上海市");

    let (st, json) = server
        .req(
            "GET",
            &format!("/warehouses/{id}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(json["data"]["code"], "WH-01");
    assert_eq!(json["data"]["name"], "主仓");
}

#[tokio::test]
async fn duplicate_warehouse_code_returns_400() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (st, _) = create_warehouse(&server, &token, "DUP-W", "A", None).await;
    assert_eq!(st, StatusCode::CREATED);

    let (st, json) = create_warehouse(&server, &token, "DUP-W", "B", None).await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "duplicate warehouse: {json}");
    assert_eq!(json["code"], 10002);
    assert_eq!(json["success"], false);
}

#[tokio::test]
async fn list_warehouses_with_filter() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    create_warehouse(&server, &token, "L-1", "北方仓", None).await;
    create_warehouse(&server, &token, "L-2", "南方仓", None).await;
    create_warehouse(&server, &token, "L-3", "东方仓", None).await;

    let (st, json) = server
        .req(
            "GET",
            "/warehouses?name=%E4%BB%93&page=1&page_size=20",
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    let items = json["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 3, "name like 仓: {json}");
    assert_eq!(json["meta"]["total"], 3);

    let (st, json) = server
        .req(
            "GET",
            "/warehouses?code=L-2&page=1&page_size=20",
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    let items = json["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["code"], "L-2");
}

#[tokio::test]
async fn update_warehouse_changes_fields() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (_, created) = create_warehouse(&server, &token, "UH-1", "旧名", Some("旧址")).await;
    let id = created["data"]["id"].as_i64().unwrap();

    let body = r#"{"code":"UH-1","name":"新名","address":"新址"}"#.to_string();
    let (st, json) = server
        .req("PUT", &format!("/warehouses/{id}"), body, Some(&token))
        .await;
    assert_eq!(st, StatusCode::OK, "update warehouse: {json}");
    assert_eq!(json["data"]["name"], "新名");
    assert_eq!(json["data"]["address"], "新址");
    assert_eq!(json["data"]["code"], "UH-1");
}

#[tokio::test]
async fn delete_warehouse_then_404() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (_, created) = create_warehouse(&server, &token, "DH-1", "删除仓", None).await;
    let id = created["data"]["id"].as_i64().unwrap();

    let (st, _) = server
        .req(
            "DELETE",
            &format!("/warehouses/{id}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);

    let (st, json) = server
        .req(
            "GET",
            &format!("/warehouses/{id}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::NOT_FOUND, "get after delete: {json}");
    assert_eq!(json["code"], 10003);
}

#[tokio::test]
async fn create_location_with_nonexistent_warehouse_returns_404() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (st, json) = create_location(&server, &token, Some(9999), "LOC-X", "库位X").await;
    assert_ne!(st, StatusCode::CREATED, "should not create: {json}");
    assert_eq!(st, StatusCode::NOT_FOUND, "fk violation: {json}");
    assert_eq!(json["code"], 10003);
}

#[tokio::test]
async fn create_and_get_location() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (_, wh) = create_warehouse(&server, &token, "WL-1", "仓库1", None).await;
    let wid = wh["data"]["id"].as_i64().unwrap();

    let (st, json) = create_location(&server, &token, Some(wid), "A-01", "A区1号位").await;
    assert_eq!(st, StatusCode::CREATED, "create location: {json}");
    let id = json["data"]["id"].as_i64().unwrap();
    assert!(id > 0);
    assert_eq!(json["data"]["code"], "A-01");
    assert_eq!(json["data"]["name"], "A区1号位");
    assert_eq!(json["data"]["warehouse_id"], wid);

    let (st, json) = server
        .req(
            "GET",
            &format!("/locations/{id}"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(json["data"]["code"], "A-01");
    assert_eq!(json["data"]["warehouse_id"], wid);
}

#[tokio::test]
async fn list_locations_with_warehouse_filter() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let (_, wh1) = create_warehouse(&server, &token, "FW-1", "仓1", None).await;
    let wid1 = wh1["data"]["id"].as_i64().unwrap();
    let (_, wh2) = create_warehouse(&server, &token, "FW-2", "仓2", None).await;
    let wid2 = wh2["data"]["id"].as_i64().unwrap();

    create_location(&server, &token, Some(wid1), "P-1", "位1").await;
    create_location(&server, &token, Some(wid1), "P-2", "位2").await;
    create_location(&server, &token, Some(wid2), "P-3", "位3").await;

    let (st, json) = server
        .req(
            "GET",
            &format!("/locations?warehouse_id={wid1}&page=1&page_size=20"),
            String::new(),
            Some(&token),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    let items = json["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 2, "warehouse_id filter: {json}");
    assert_eq!(json["meta"]["total"], 2);
    for it in items {
        assert_eq!(it["warehouse_id"], wid1);
    }
}

#[tokio::test]
async fn rbac_finrole_forbidden_manager_readonly() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let admin = admin_token(&server).await;

    // finance 角色 (role_id=6) 无 stock.read / stock.write
    let body = r#"{"username":"fin","password":"pass1234","display_name":"Fin","role_ids":[6]}"#
        .to_string();
    let (st, json) = server.req("POST", "/users", body, Some(&admin)).await;
    assert_eq!(st, StatusCode::CREATED, "create finance user: {json}");

    let body = r#"{"username":"fin","password":"pass1234"}"#.to_string();
    let (st, json) = server.req("POST", "/auth/login", body, None).await;
    assert_eq!(st, StatusCode::OK, "finance login: {json}");
    let fin_token = json["data"]["access_token"].as_str().unwrap().to_string();

    // finance: 无 stock.read → GET /warehouses 403
    let (st, json) = server
        .req(
            "GET",
            "/warehouses?page=1&page_size=20",
            String::new(),
            Some(&fin_token),
        )
        .await;
    assert_eq!(st, StatusCode::FORBIDDEN, "finance get warehouses: {json}");
    assert_eq!(json["code"], 11003);

    // manager 角色 (role_id=2) 有 stock.read, 无 stock.write
    let body = r#"{"username":"mgr","password":"pass1234","display_name":"Mgr","role_ids":[2]}"#
        .to_string();
    let (st, json) = server.req("POST", "/users", body, Some(&admin)).await;
    assert_eq!(st, StatusCode::CREATED, "create manager: {json}");
    let body = r#"{"username":"mgr","password":"pass1234"}"#.to_string();
    let (st, json) = server.req("POST", "/auth/login", body, None).await;
    assert_eq!(st, StatusCode::OK, "manager login: {json}");
    let mgr_token = json["data"]["access_token"].as_str().unwrap().to_string();

    // manager: 有 stock.read → GET 200
    let (st, _) = server
        .req(
            "GET",
            "/warehouses?page=1&page_size=20",
            String::new(),
            Some(&mgr_token),
        )
        .await;
    assert_eq!(st, StatusCode::OK, "manager get warehouses should be 200");

    // manager: 无 stock.write → POST 403
    let body = r#"{"code":"RBAC-W","name":"X"}"#.to_string();
    let (st, json) = server
        .req("POST", "/warehouses", body, Some(&mgr_token))
        .await;
    assert_eq!(st, StatusCode::FORBIDDEN, "manager post warehouse: {json}");
    assert_eq!(json["code"], 11003);
}

#[tokio::test]
async fn empty_code_or_name_returns_400() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;
    let token = admin_token(&server).await;

    let body = r#"{"code":"","name":"NoCode"}"#.to_string();
    let (st, json) = server.req("POST", "/warehouses", body, Some(&token)).await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "empty code: {json}");
    assert_eq!(json["code"], 10002);

    let body = r#"{"code":"HAS","name":"   "}"#.to_string();
    let (st, json) = server.req("POST", "/warehouses", body, Some(&token)).await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "blank name: {json}");
    assert_eq!(json["code"], 10002);
}
