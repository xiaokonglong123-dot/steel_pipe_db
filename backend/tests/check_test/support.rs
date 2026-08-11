use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use sqlx::SqlitePool;
use tempfile::TempDir;
use tower::ServiceExt;

use super::common::test_pool;
use erp_v2::auth::bootstrap_admin;
use erp_v2::config::Config;
use erp_v2::http::router;

pub struct TestServer {
    pub app: Router,
    pub pool: SqlitePool,
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
            app: router(pool.clone(), "test-secret".to_string()),
            pool,
            _dir: dir,
        }
    }

    pub async fn req(
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

pub async fn fixture() -> (TestServer, String, i64, i64) {
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
    let (item_status, item) = server
        .req(
            "POST",
            "/items",
            r#"{"sku":"CHECK-SKU","name":"盘点商品"}"#,
            Some(&token),
        )
        .await;
    assert_eq!(item_status, StatusCode::CREATED, "create item: {item}");
    let item_id = item["data"]["id"].as_i64().unwrap();
    let (location_status, location) = server
        .req(
            "POST",
            "/locations",
            r#"{"code":"CHECK-LOC","name":"盘点库位"}"#,
            Some(&token),
        )
        .await;
    assert_eq!(
        location_status,
        StatusCode::CREATED,
        "create location: {location}"
    );
    (
        server,
        token,
        item_id,
        location["data"]["id"].as_i64().unwrap(),
    )
}

pub async fn receive(server: &TestServer, token: &str, item_id: i64, location_id: i64) {
    let body = format!(
        r#"{{"inbound_type":"purchase","items":[{{"item_id":{item_id},"location_id":{location_id},"quantity":100}}]}}"#
    );
    let (status, created) = server.req("POST", "/inbounds", &body, Some(token)).await;
    assert_eq!(status, StatusCode::CREATED, "create inbound: {created}");
    let inbound_id = created["data"]["id"].as_i64().unwrap();
    let (status, posted) = server
        .req(
            "POST",
            &format!("/inbounds/{inbound_id}/post"),
            "",
            Some(token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "post inbound: {posted}");
}

pub async fn create_check(server: &TestServer, token: &str, location_id: i64) -> (i64, i64) {
    let body = format!(r#"{{"location_id":{location_id},"scope":"all"}}"#);
    let (status, created) = server
        .req("POST", "/check-records", &body, Some(token))
        .await;
    assert_eq!(status, StatusCode::CREATED, "create check: {created}");
    let session_id = created["data"]["id"].as_i64().unwrap();
    let (status, detail) = server
        .req(
            "GET",
            &format!("/check-records/{session_id}"),
            "",
            Some(token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "get check: {detail}");
    let detail_id = detail["data"]["details"][0]["id"].as_i64().unwrap();
    (session_id, detail_id)
}
