//! M3 端到端：入库合规门禁、出库守卫、批量原子、调拨、质量标注、目录 CRUD。
//!
//! 走真实 Router + 真实 SQLite；断言重点在 HTTP 状态与「副作用是否发生」。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, Response, StatusCode, header};
use http_body_util::BodyExt;
use steelpipe_api::{AppState, router};
use steelpipe_auth::{Keys, hash_password};
use steelpipe_domain::Role;
use steelpipe_infra::repo::users;
use steelpipe_infra::{DbPool, time, uow::Uow};
use steelpipe_service::{AuthService, CatalogService, Identity, UserService};
use tower::ServiceExt;

struct Harness {
    app: Router,
    pool: DbPool,
    path: PathBuf,
    admin_cookies: String,
    operator_cookies: String,
    admin_csrf: String,
    operator_csrf: String,
}

impl Harness {
    async fn new(tag: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("steelpipe-invapi-{tag}-{nanos}.db"));
        let pool = steelpipe_infra::connect(&format!("sqlite://{}", path.display()), 8)
            .await
            .unwrap();
        steelpipe_infra::migrate(&pool).await.unwrap();

        // 种子数据：两个库位 + L80 规格 + 两个账号
        let catalog = CatalogService::new(pool.clone());
        let bootstrap_actor = Identity {
            user_id: 0,
            username: "system".into(),
            display_name: "system".into(),
            role: Role::Admin,
            token_version: 0,
        };
        catalog
            .create_location(&bootstrap_actor, "A1", Some("一号库"), Some(100))
            .await
            .unwrap();
        catalog
            .create_location(&bootstrap_actor, "A2", Some("二号库"), None)
            .await
            .unwrap();
        catalog
            .create_standard(
                &bootstrap_actor,
                steelpipe_service::StandardInput {
                    grade: "L80".into(),
                    outer_diameter_mm: Some(88.9),
                    min_wall_thickness_mm: Some(6.45),
                    max_wall_thickness_mm: Some(7.34),
                    length_tolerance_mm: None,
                    source: Some("test".into()),
                },
            )
            .await
            .unwrap();

        let hash = hash_password("pw-123456").unwrap();
        let mut uow = Uow::begin(&pool).await.unwrap();
        let _admin_id = users::insert(
            uow.conn(),
            "root",
            &hash,
            "Root",
            Role::Admin,
            &time::now_rfc3339(),
        )
        .await
        .unwrap();
        let _operator_id = users::insert(
            uow.conn(),
            "op",
            &hash,
            "Op",
            Role::Operator,
            &time::now_rfc3339(),
        )
        .await
        .unwrap();
        uow.commit().await.unwrap();

        let keys = Keys::from_secret(b"api-inventory-test-secret-32byte!");
        let auth = Arc::new(AuthService::new(pool.clone(), keys));
        let users_service = Arc::new(UserService::new(pool.clone()));
        let state = AppState::new(auth, users_service, None, false, "test");
        let app = router(state);

        let mut harness = Self {
            app,
            pool,
            path,
            admin_cookies: String::new(),
            operator_cookies: String::new(),
            admin_csrf: String::new(),
            operator_csrf: String::new(),
        };
        harness.admin_cookies = harness.login("root").await;
        harness.operator_cookies = harness.login("op").await;
        harness.admin_csrf = harness
            .cookies_map(&harness.admin_cookies)
            .get("csrf_token")
            .cloned()
            .unwrap_or_default();
        harness.operator_csrf = harness
            .cookies_map(&harness.operator_cookies)
            .get("csrf_token")
            .cloned()
            .unwrap_or_default();
        harness
    }

    async fn login(&self, username: &str) -> String {
        let csrf = self.send(get("/api/auth/csrf"), "").await;
        let token = csrf.json()["csrf_token"].as_str().unwrap().to_owned();
        let cookies = csrf.cookie_header(&["csrf_token"]);
        let reply = self
            .send(
                with_csrf(
                    with_cookies(
                        post_json(
                            "/api/auth/login",
                            serde_json::json!({"username": username, "password": "pw-123456"}),
                        ),
                        &cookies,
                    ),
                    &token,
                ),
                "",
            )
            .await;
        assert_eq!(reply.status, StatusCode::OK, "登录失败：{}", reply.body);
        reply.cookie_header(&["access_token", "refresh_token", "csrf_token"])
    }

    fn cookies_map(&self, cookies: &str) -> HashMap<String, String> {
        cookies
            .split(';')
            .filter_map(|pair| pair.trim().split_once('='))
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect()
    }

    async fn send(&self, request: Request<Body>, _unused: &str) -> Reply {
        let mut request = request;
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 42000))));
        let response = self.app.clone().oneshot(request).await.expect("请求失败");
        Reply::from_response(response).await
    }

    async fn call(&self, request: Request<Body>, cookies: &str, csrf: Option<&str>) -> Reply {
        let mut request = with_cookies(request, cookies);
        if let Some(token) = csrf {
            request = with_csrf(request, token);
        }
        self.send(request, "").await
    }

    async fn cleanup(self) {
        self.pool.close().await;
        let _ = std::fs::remove_file(&self.path);
    }
}

struct Reply {
    status: StatusCode,
    body: String,
    cookies: HashMap<String, String>,
}

impl Reply {
    async fn from_response(response: Response<Body>) -> Self {
        let status = response.status();
        let headers = response.headers().clone();
        let cookies = headers
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .filter_map(|value| value.split(';').next())
            .filter_map(|pair| pair.split_once('='))
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("读取响应体失败")
            .to_bytes();
        let _ = headers;
        Self {
            status,
            body: String::from_utf8_lossy(&bytes).to_string(),
            cookies,
        }
    }

    fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body).unwrap_or(serde_json::Value::Null)
    }

    fn cookie_header(&self, names: &[&str]) -> String {
        names
            .iter()
            .filter_map(|name| {
                self.cookies
                    .get(*name)
                    .map(|value| format!("{name}={value}"))
            })
            .collect::<Vec<_>>()
            .join("; ")
    }
}

fn get(path: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .body(Body::empty())
        .unwrap()
}

fn delete(path: &str) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(path)
        .body(Body::empty())
        .unwrap()
}

fn post_json(path: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn put_json(path: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn with_cookies(mut request: Request<Body>, cookies: &str) -> Request<Body> {
    if !cookies.is_empty() {
        request
            .headers_mut()
            .insert(header::COOKIE, cookies.parse().expect("cookie 头不合法"));
    }
    request
}

fn with_csrf(mut request: Request<Body>, token: &str) -> Request<Body> {
    request
        .headers_mut()
        .insert("x-csrf-token", token.parse().expect("csrf 头不合法"));
    request
}

fn stock_in_body(pipe_id: &str, wall: f64) -> serde_json::Value {
    serde_json::json!({
        "pipe_id": pipe_id,
        "pipe_type": "seamless_pipe",
        "outer_diameter_mm": 88.9,
        "wall_thickness_mm": wall,
        "length_m": 9.5,
        "steel_grade": "L80",
        "thread_type": "EUE",
        "manufacturer": "宝钢",
        "location": "A1",
        "quality_mark": "normal",
        "seamless": {"surface_treatment": "磷化"}
    })
}

#[tokio::test]
async fn stock_in_then_list_and_detail() {
    let fx = Harness::new("in").await;
    let reply = fx
        .call(
            post_json("/api/tubes/stock-in", stock_in_body("P-100", 7.0)),
            &fx.operator_cookies,
            Some(&fx.operator_csrf),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "body: {}", reply.body);
    assert_eq!(reply.json()["compliance_pass"], true);

    let list = fx.call(get("/api/tubes"), &fx.operator_cookies, None).await;
    assert_eq!(list.status, StatusCode::OK);
    assert_eq!(list.json()["total"], 1);

    let detail = fx
        .call(get("/api/tubes/P-100"), &fx.operator_cookies, None)
        .await;
    assert_eq!(detail.status, StatusCode::OK);
    assert_eq!(detail.json()["tube"]["pipe_id"], "P-100");
    assert_eq!(detail.json()["seamless"]["surface_treatment"], "磷化");
    assert_eq!(detail.json()["history"].as_array().unwrap().len(), 1);

    // used_count 反映在库位列表
    let locations = fx
        .call(get("/api/locations"), &fx.operator_cookies, None)
        .await;
    let a1 = locations.json()["locations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["code"] == "A1")
        .cloned()
        .unwrap();
    assert_eq!(a1["used_count"], 1);

    fx.cleanup().await;
}

#[tokio::test]
async fn non_compliant_stock_in_is_422_and_force_requires_admin() {
    let fx = Harness::new("reject").await;
    let reply = fx
        .call(
            post_json("/api/tubes/stock-in", stock_in_body("P-101", 3.0)),
            &fx.operator_cookies,
            Some(&fx.operator_csrf),
        )
        .await;
    assert_eq!(
        reply.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "body: {}",
        reply.body
    );
    assert!(
        reply.json()["detail"]
            .as_str()
            .unwrap()
            .contains("wall_thickness_range")
    );

    let mut forced = stock_in_body("P-101", 3.0);
    forced["force"] = serde_json::json!(true);
    let forbidden = fx
        .call(
            post_json("/api/tubes/stock-in", forced.clone()),
            &fx.operator_cookies,
            Some(&fx.operator_csrf),
        )
        .await;
    assert_eq!(forbidden.status, StatusCode::FORBIDDEN);

    let admin_ok = fx
        .call(
            post_json("/api/tubes/stock-in", forced),
            &fx.admin_cookies,
            Some(&fx.admin_csrf),
        )
        .await;
    assert_eq!(
        admin_ok.status,
        StatusCode::CREATED,
        "body: {}",
        admin_ok.body
    );
    assert_eq!(admin_ok.json()["forced"], true);

    fx.cleanup().await;
}

#[tokio::test]
async fn stock_out_twice_is_409_and_batch_is_atomic() {
    let fx = Harness::new("out").await;
    for pipe_id in ["P-102", "P-103"] {
        fx.call(
            post_json("/api/tubes/stock-in", stock_in_body(pipe_id, 7.0)),
            &fx.operator_cookies,
            Some(&fx.operator_csrf),
        )
        .await;
    }

    let first = fx
        .call(
            post_json(
                "/api/tubes/P-102/stock-out",
                serde_json::json!({"remark": "发货"}),
            ),
            &fx.operator_cookies,
            Some(&fx.operator_csrf),
        )
        .await;
    assert_eq!(first.status, StatusCode::OK, "body: {}", first.body);

    let second = fx
        .call(
            post_json("/api/tubes/P-102/stock-out", serde_json::json!({})),
            &fx.operator_cookies,
            Some(&fx.operator_csrf),
        )
        .await;
    assert_eq!(second.status, StatusCode::CONFLICT);

    // 批量包含已出库的 P-102 → 409，且 P-103 不得被出库
    let batch = fx
        .call(
            post_json(
                "/api/tubes/stock-out/batch",
                serde_json::json!({"pipe_ids": ["P-103", "P-102"]}),
            ),
            &fx.operator_cookies,
            Some(&fx.operator_csrf),
        )
        .await;
    assert_eq!(batch.status, StatusCode::CONFLICT, "body: {}", batch.body);

    let detail = fx
        .call(get("/api/tubes/P-103"), &fx.operator_cookies, None)
        .await;
    assert_eq!(detail.json()["tube"]["status"], "in_stock");

    fx.cleanup().await;
}

#[tokio::test]
async fn transfer_moves_tube_and_rejects_same_location() {
    let fx = Harness::new("transfer").await;
    fx.call(
        post_json("/api/tubes/stock-in", stock_in_body("P-104", 7.0)),
        &fx.operator_cookies,
        Some(&fx.operator_csrf),
    )
    .await;

    let same = fx
        .call(
            post_json(
                "/api/tubes/P-104/transfer",
                serde_json::json!({"target_location": "A1"}),
            ),
            &fx.operator_cookies,
            Some(&fx.operator_csrf),
        )
        .await;
    assert_eq!(same.status, StatusCode::BAD_REQUEST);

    let moved = fx
        .call(
            post_json(
                "/api/tubes/P-104/transfer",
                serde_json::json!({"target_location": "A2"}),
            ),
            &fx.operator_cookies,
            Some(&fx.operator_csrf),
        )
        .await;
    assert_eq!(moved.status, StatusCode::OK, "body: {}", moved.body);
    assert_eq!(moved.json()["to"], "A2");

    let detail = fx
        .call(get("/api/tubes/P-104"), &fx.operator_cookies, None)
        .await;
    assert_eq!(detail.json()["tube"]["location"], "A2");

    fx.cleanup().await;
}

#[tokio::test]
async fn quality_mark_and_soft_delete_are_audited_and_filtered() {
    let fx = Harness::new("quality").await;
    fx.call(
        post_json("/api/tubes/stock-in", stock_in_body("P-105", 7.0)),
        &fx.operator_cookies,
        Some(&fx.operator_csrf),
    )
    .await;

    let marked = fx
        .call(
            post_json(
                "/api/tubes/P-105/quality",
                serde_json::json!({"quality_mark": "problematic", "note": "壁厚超差"}),
            ),
            &fx.operator_cookies,
            Some(&fx.operator_csrf),
        )
        .await;
    assert_eq!(marked.status, StatusCode::NO_CONTENT);

    // 问题管出库需要 admin
    let blocked = fx
        .call(
            post_json("/api/tubes/P-105/stock-out", serde_json::json!({})),
            &fx.operator_cookies,
            Some(&fx.operator_csrf),
        )
        .await;
    assert_eq!(
        blocked.status,
        StatusCode::CONFLICT,
        "body: {}",
        blocked.body
    );

    // 软删：operator 无权（admin only）
    let operator_delete = fx
        .call(
            delete("/api/tubes/P-105"),
            &fx.operator_cookies,
            Some(&fx.operator_csrf),
        )
        .await;
    assert_eq!(operator_delete.status, StatusCode::FORBIDDEN);

    let deleted = fx
        .call(
            delete("/api/tubes/P-105"),
            &fx.admin_cookies,
            Some(&fx.admin_csrf),
        )
        .await;
    assert_eq!(
        deleted.status,
        StatusCode::NO_CONTENT,
        "body: {}",
        deleted.body
    );

    let list = fx.call(get("/api/tubes"), &fx.admin_cookies, None).await;
    assert_eq!(list.json()["total"], 0, "软删后默认列表应为空");
    let with_deleted = fx
        .call(
            get("/api/tubes?include_deleted=true"),
            &fx.admin_cookies,
            None,
        )
        .await;
    assert_eq!(with_deleted.json()["total"], 1);

    let actions: Vec<String> = steelpipe_infra::repo::audit::page_after(&fx.pool, 0, 200)
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.action)
        .collect();
    assert!(actions.contains(&"quality_mark".to_owned()));
    assert!(actions.contains(&"delete_tube".to_owned()));

    fx.cleanup().await;
}

#[tokio::test]
async fn catalog_write_paths_require_admin() {
    let fx = Harness::new("catalog").await;

    let operator_create = fx
        .call(
            post_json(
                "/api/locations",
                serde_json::json!({"code": "B1", "name": "B1"}),
            ),
            &fx.operator_cookies,
            Some(&fx.operator_csrf),
        )
        .await;
    assert_eq!(operator_create.status, StatusCode::FORBIDDEN);

    let admin_create = fx
        .call(
            post_json(
                "/api/locations",
                serde_json::json!({"code": "B1", "name": "B1", "capacity": 20}),
            ),
            &fx.admin_cookies,
            Some(&fx.admin_csrf),
        )
        .await;
    assert_eq!(
        admin_create.status,
        StatusCode::CREATED,
        "body: {}",
        admin_create.body
    );

    let duplicate = fx
        .call(
            post_json("/api/locations", serde_json::json!({"code": "B1"})),
            &fx.admin_cookies,
            Some(&fx.admin_csrf),
        )
        .await;
    assert_eq!(duplicate.status, StatusCode::CONFLICT);

    let deactivate = fx
        .call(
            post_json(
                "/api/locations/B1/active",
                serde_json::json!({"active": false}),
            ),
            &fx.admin_cookies,
            Some(&fx.admin_csrf),
        )
        .await;
    assert_eq!(deactivate.status, StatusCode::NO_CONTENT);

    // 停用库位不能入库
    let mut body = stock_in_body("P-106", 7.0);
    body["location"] = serde_json::json!("B1");
    let rejected = fx
        .call(
            post_json("/api/tubes/stock-in", body),
            &fx.admin_cookies,
            Some(&fx.admin_csrf),
        )
        .await;
    assert_eq!(rejected.status, StatusCode::BAD_REQUEST);

    // 标准规格：创建 → 改名冲突 → 删除
    let created = fx
        .call(
            post_json(
                "/api/standards",
                serde_json::json!({"grade": "N80", "min_wall_thickness_mm": 6.45, "max_wall_thickness_mm": 7.34}),
            ),
            &fx.admin_cookies,
            Some(&fx.admin_csrf),
        )
        .await;
    assert_eq!(
        created.status,
        StatusCode::CREATED,
        "body: {}",
        created.body
    );
    let id = created.json()["id"].as_i64().unwrap();

    let conflict = fx
        .call(
            put_json(
                &format!("/api/standards/{id}"),
                serde_json::json!({"grade": "L80"}),
            ),
            &fx.admin_cookies,
            Some(&fx.admin_csrf),
        )
        .await;
    assert_eq!(conflict.status, StatusCode::CONFLICT);

    let deleted = fx
        .call(
            delete(&format!("/api/standards/{id}")),
            &fx.admin_cookies,
            Some(&fx.admin_csrf),
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);

    let standards = fx
        .call(get("/api/standards"), &fx.admin_cookies, None)
        .await;
    assert_eq!(standards.json()["total"], 1);

    fx.cleanup().await;
}

fn post_csv(path: &str, csv: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header(header::CONTENT_TYPE, "text/csv")
        .body(Body::from(csv.to_owned()))
        .unwrap()
}

#[tokio::test]
async fn csv_import_is_row_level_partial_success() {
    let fx = Harness::new("import").await;
    let csv = "pipe_id,pipe_type,outer_diameter_mm,wall_thickness_mm,length_m,steel_grade,manufacturer,location\n\
               P-200,seamless_pipe,88.9,7.0,9.5,L80,宝钢,A1\n\
               P-201,seamless_pipe,88.9,3.0,9.5,L80,宝钢,A1\n\
               P-202,seamless_pipe,88.9,7.0,9.5,X99,宝钢,A1\n";
    let reply = fx
        .call(
            post_csv("/api/tubes/import", csv),
            &fx.operator_cookies,
            Some(&fx.operator_csrf),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "body: {}", reply.body);
    assert_eq!(reply.json()["total"], 3);
    assert_eq!(reply.json()["inserted"], 1);
    assert_eq!(reply.json()["failed"].as_array().unwrap().len(), 2);

    let list = fx.call(get("/api/tubes"), &fx.operator_cookies, None).await;
    assert_eq!(list.json()["total"], 1);

    // 缺列 → 400，且不写任何行
    let bad = fx
        .call(
            post_csv("/api/tubes/import", "pipe_id\nP-300\n"),
            &fx.operator_cookies,
            Some(&fx.operator_csrf),
        )
        .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    let list = fx.call(get("/api/tubes"), &fx.operator_cookies, None).await;
    assert_eq!(list.json()["total"], 1, "失败导入不得新增");

    fx.cleanup().await;
}

#[tokio::test]
async fn import_requires_csrf_and_operator() {
    let fx = Harness::new("import-rbac").await;
    let csv = "pipe_id,pipe_type,outer_diameter_mm,wall_thickness_mm,length_m,steel_grade,manufacturer,location\nP-300,seamless_pipe,88.9,7.0,9.5,L80,宝钢,A1\n";

    let no_csrf = fx
        .call(
            post_csv("/api/tubes/import", csv),
            &fx.operator_cookies,
            None,
        )
        .await;
    assert_eq!(no_csrf.status, StatusCode::FORBIDDEN);

    let no_auth = fx
        .call(
            post_csv("/api/tubes/import", csv),
            "",
            Some(&fx.operator_csrf),
        )
        .await;
    assert_eq!(no_auth.status, StatusCode::UNAUTHORIZED);

    fx.cleanup().await;
}
