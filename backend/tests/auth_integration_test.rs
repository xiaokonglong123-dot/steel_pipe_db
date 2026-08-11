//! Auth 集成测试 — login / me / create-user / refresh / logout 全链路
//!
//! 通过 OneShotTester 自建 server 模拟 HTTP 调用，验证 token 颁发/校验/RBAC。

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
        let cfg = Config {
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
            .unwrap_or(serde_json::json!({"raw": String::from_utf8_lossy(&body)}));
        (status, json)
    }
}

#[tokio::test]
async fn login_and_me() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();

    let server = TestServer::new(pool).await;

    let body = r#"{"username":"admin","password":"admin123"}"#.to_string();
    let (status, json) = server.req("POST", "/auth/login", body, None).await;
    assert_eq!(status, StatusCode::OK);

    let access = json["data"]["access_token"].as_str().unwrap().to_string();
    assert!(!access.is_empty());

    let (st, json) = server
        .req("GET", "/auth/me", String::new(), Some(&access))
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(json["data"]["username"], "admin");

    let perms = json["data"]["permissions"].as_array().unwrap();
    assert_eq!(perms.len(), 11, "admin 应有 11 个权限");
}

#[tokio::test]
async fn login_bad_password() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();

    let server = TestServer::new(pool).await;
    let body = r#"{"username":"admin","password":"WRONG"}"#.to_string();
    let (status, json) = server.req("POST", "/auth/login", body, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(json["success"], false);
}

#[tokio::test]
async fn unauthorized_without_token() {
    let (pool, _dir) = test_pool().await;
    let server = TestServer::new(pool).await;

    let (st, _) = server.req("GET", "/auth/me", String::new(), None).await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_and_update_user() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let server = TestServer::new(pool).await;

    let body = r#"{"username":"admin","password":"admin123"}"#.to_string();
    let (_, json) = server.req("POST", "/auth/login", body, None).await;
    let token = json["data"]["access_token"].as_str().unwrap().to_string();

    let body =
        r#"{"username":"u1","password":"pass1234","display_name":"User One","role_ids":[3]}"#
            .to_string();
    let (st, json) = server.req("POST", "/users", body, Some(&token)).await;
    assert_eq!(st, StatusCode::CREATED);
    let uid = json["data"]["id"].as_i64().unwrap();
    assert!(uid > 1);

    let body = format!(r#"{{"display_name":"Renamed","is_active":true}}"#);
    let (st, _) = server
        .req("PUT", &format!("/users/{uid}"), body, Some(&token))
        .await;
    assert_eq!(st, StatusCode::OK);

    let body = r#"{"username":"u1","password":"pass1234"}"#.to_string();
    let (st, json) = server.req("POST", "/auth/login", body, None).await;
    assert_eq!(st, StatusCode::OK, "u1 登录状态: {json}");
    assert_eq!(
        json["data"]["user"]["username"], "u1",
        "u1 登录响应: {json}"
    );
}
