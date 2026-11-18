//! M4 端到端：盘点开单、逐根核对、盘盈合规、确认原子性、找回、取消。

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
use steelpipe_domain::{QualityMark, Role, TubeType};
use steelpipe_infra::repo::users;
use steelpipe_infra::{DbPool, time, uow::Uow};
use steelpipe_service::{
    AuthService, CatalogService, Identity, InventoryService, StockInInput, UserService,
};
use tower::ServiceExt;

struct Harness {
    app: Router,
    pool: DbPool,
    path: PathBuf,
    admin_cookies: String,
    admin_csrf: String,
    operator_cookies: String,
    operator_csrf: String,
}

async fn fixture(tag: &str) -> Harness {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("steelpipe-checkapi-{tag}-{nanos}.db"));
    let pool = steelpipe_infra::connect(&format!("sqlite://{}", path.display()), 8)
        .await
        .unwrap();
    steelpipe_infra::migrate(&pool).await.unwrap();

    let catalog = CatalogService::new(pool.clone());
    let system = Identity {
        user_id: 0,
        username: "system".into(),
        display_name: "system".into(),
        role: Role::Admin,
        token_version: 0,
    };
    for code in ["A1", "A2"] {
        catalog
            .create_location(&system, code, Some(code), None)
            .await
            .unwrap();
    }
    catalog
        .create_standard(
            &system,
            steelpipe_service::StandardInput {
                grade: "L80".into(),
                outer_diameter_mm: Some(88.9),
                min_wall_thickness_mm: Some(6.45),
                max_wall_thickness_mm: Some(7.34),
                length_tolerance_mm: None,
                source: None,
            },
        )
        .await
        .unwrap();

    let hash = hash_password("pw-123456").unwrap();
    let mut uow = Uow::begin(&pool).await.unwrap();
    users::insert(
        uow.conn(),
        "root",
        &hash,
        "Root",
        Role::Admin,
        &time::now_rfc3339(),
    )
    .await
    .unwrap();
    users::insert(
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

    let keys = Keys::from_secret(b"api-checks-test-secret-32-bytes!!");
    let auth = Arc::new(AuthService::new(pool.clone(), keys));
    let users_service = Arc::new(UserService::new(pool.clone()));
    let app = router(AppState::new(auth, users_service, None, false, "test"));

    let mut harness = Harness {
        app,
        pool,
        path,
        admin_cookies: String::new(),
        admin_csrf: String::new(),
        operator_cookies: String::new(),
        operator_csrf: String::new(),
    };
    harness.admin_cookies = harness.login("root").await;
    harness.operator_cookies = harness.login("op").await;
    harness.admin_csrf = harness.csrf_of(&harness.admin_cookies.clone());
    harness.operator_csrf = harness.csrf_of(&harness.operator_cookies.clone());
    harness
}

impl Harness {
    fn csrf_of(&self, cookies: &str) -> String {
        cookies
            .split(';')
            .filter_map(|pair| pair.trim().split_once('='))
            .find(|(name, _)| *name == "csrf_token")
            .map(|(_, value)| value.to_owned())
            .unwrap_or_default()
    }

    async fn login(&self, username: &str) -> String {
        let reply = self.send(get("/api/auth/csrf"), "").await;
        let token = reply.json()["csrf_token"].as_str().unwrap().to_owned();
        let reply = self
            .send(
                with_csrf(
                    with_cookies(
                        post_json(
                            "/api/auth/login",
                            serde_json::json!({"username": username, "password": "pw-123456"}),
                        ),
                        &reply.cookie_header(&["csrf_token"]),
                    ),
                    &token,
                ),
                "",
            )
            .await;
        assert_eq!(reply.status, StatusCode::OK, "登录失败：{}", reply.body);
        reply.cookie_header(&["access_token", "refresh_token", "csrf_token"])
    }

    async fn send(&self, request: Request<Body>, _unused: &str) -> Reply {
        let mut request = request;
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 43000))));
        let response = self.app.clone().oneshot(request).await.expect("请求失败");
        Reply::from_response(response).await
    }

    async fn as_admin(&self, request: Request<Body>) -> Reply {
        self.send(
            with_csrf(with_cookies(request, &self.admin_cookies), &self.admin_csrf),
            "",
        )
        .await
    }

    async fn as_operator(&self, request: Request<Body>) -> Reply {
        self.send(
            with_csrf(
                with_cookies(request, &self.operator_cookies),
                &self.operator_csrf,
            ),
            "",
        )
        .await
    }

    async fn seed_tube(&self, pipe_id: &str) {
        let inventory = InventoryService::new(self.pool.clone());
        let actor = Identity {
            user_id: 0,
            username: "system".into(),
            display_name: "system".into(),
            role: Role::Admin,
            token_version: 0,
        };
        inventory
            .stock_in(
                &actor,
                StockInInput {
                    pipe_id: pipe_id.into(),
                    pipe_type: TubeType::SeamlessPipe,
                    outer_diameter_mm: 88.9,
                    wall_thickness_mm: 7.0,
                    length_m: 9.5,
                    steel_grade: "L80".into(),
                    thread_type: None,
                    api_5ct_cert: None,
                    manufacturer: "宝钢".into(),
                    batch_no: None,
                    heat_no: None,
                    heat_treat_method: None,
                    location: "A1".into(),
                    quality_mark: QualityMark::Normal,
                    quality_note: None,
                    screen: None,
                    seamless: None,
                    force: false,
                },
            )
            .await
            .expect("种子入库失败");
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
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
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

fn post_json(path: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
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

fn surplus_body(pipe_id: &str, wall: f64) -> serde_json::Value {
    serde_json::json!({
        "pipe_id": pipe_id,
        "pipe_type": "seamless_pipe",
        "outer_diameter_mm": 88.9,
        "wall_thickness_mm": wall,
        "length_m": 9.5,
        "steel_grade": "L80",
        "manufacturer": "宝钢",
        "location": "A1",
        "quality_mark": "normal"
    })
}

#[tokio::test]
async fn full_check_flow_applies_differences() {
    let fx = fixture("flow").await;
    fx.seed_tube("P-1").await;
    fx.seed_tube("P-2").await;

    let created = fx
        .as_operator(post_json(
            "/api/checks",
            serde_json::json!({"location": "A1", "remark": "月盘"}),
        ))
        .await;
    assert_eq!(
        created.status,
        StatusCode::CREATED,
        "body: {}",
        created.body
    );
    let check_id = created.json()["id"].as_i64().unwrap();

    let detail = fx
        .as_operator(get(&format!("/api/checks/{check_id}")))
        .await;
    assert_eq!(detail.json()["expected"].as_array().unwrap().len(), 2);

    for (pipe_id, result, actual) in [
        ("P-1", "missing", serde_json::Value::Null),
        ("P-2", "moved", serde_json::json!("A2")),
    ] {
        let response = fx
            .as_operator(post_json(
                &format!("/api/checks/{check_id}/items"),
                serde_json::json!({"pipe_id": pipe_id, "result": result, "actual_location": actual}),
            ))
            .await;
        assert_eq!(
            response.status,
            StatusCode::NO_CONTENT,
            "body: {}",
            response.body
        );
    }

    // operator 不能确认（设计：管理员 confirm）
    let forbidden = fx
        .as_operator(post_json(
            &format!("/api/checks/{check_id}/confirm"),
            serde_json::json!({}),
        ))
        .await;
    assert_eq!(forbidden.status, StatusCode::FORBIDDEN);

    let confirmed = fx
        .as_admin(post_json(
            &format!("/api/checks/{check_id}/confirm"),
            serde_json::json!({}),
        ))
        .await;
    assert_eq!(confirmed.status, StatusCode::OK, "body: {}", confirmed.body);
    assert_eq!(confirmed.json()["missing"], 1);
    assert_eq!(confirmed.json()["moved"], 1);

    let p1 = fx.as_operator(get("/api/tubes/P-1")).await;
    assert_eq!(p1.json()["tube"]["status"], "missing");
    let p2 = fx.as_operator(get("/api/tubes/P-2")).await;
    assert_eq!(p2.json()["tube"]["location"], "A2");

    let locations = fx.as_operator(get("/api/locations")).await;
    let a1 = locations.json()["locations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["code"] == "A1")
        .cloned()
        .unwrap();
    assert_eq!(a1["used_count"], 0);

    // 找回
    let recovered = fx
        .as_operator(post_json("/api/tubes/P-1/recover", serde_json::json!({})))
        .await;
    assert_eq!(
        recovered.status,
        StatusCode::NO_CONTENT,
        "body: {}",
        recovered.body
    );
    let p1 = fx.as_operator(get("/api/tubes/P-1")).await;
    assert_eq!(p1.json()["tube"]["status"], "in_stock");

    fx.cleanup().await;
}

#[tokio::test]
async fn surplus_is_gated_by_compliance_at_confirm_time() {
    let fx = fixture("surplus").await;
    let created = fx
        .as_operator(post_json(
            "/api/checks",
            serde_json::json!({"location": "A1"}),
        ))
        .await;
    let check_id = created.json()["id"].as_i64().unwrap();

    let bad = fx
        .as_operator(post_json(
            &format!("/api/checks/{check_id}/surplus"),
            surplus_body("P-9", 3.0),
        ))
        .await;
    assert_eq!(bad.status, StatusCode::NO_CONTENT, "登记阶段不校验合规");

    let rejected = fx
        .as_admin(post_json(
            &format!("/api/checks/{check_id}/confirm"),
            serde_json::json!({}),
        ))
        .await;
    assert_eq!(
        rejected.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "body: {}",
        rejected.body
    );

    let detail = fx
        .as_operator(get(&format!("/api/checks/{check_id}")))
        .await;
    assert_eq!(
        detail.json()["session"]["status"],
        "open",
        "失败必须保持 open"
    );
    let item_id = detail.json()["items"][0]["id"].as_i64().unwrap();

    let removed = fx
        .as_operator(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/checks/{check_id}/items/{item_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);

    let good = fx
        .as_operator(post_json(
            &format!("/api/checks/{check_id}/surplus"),
            surplus_body("P-9", 7.0),
        ))
        .await;
    assert_eq!(good.status, StatusCode::NO_CONTENT);

    let confirmed = fx
        .as_admin(post_json(
            &format!("/api/checks/{check_id}/confirm"),
            serde_json::json!({}),
        ))
        .await;
    assert_eq!(confirmed.status, StatusCode::OK, "body: {}", confirmed.body);
    assert_eq!(confirmed.json()["surplus"], 1);

    let tube = fx.as_operator(get("/api/tubes/P-9")).await;
    assert_eq!(tube.status, StatusCode::OK);
    assert_eq!(tube.json()["tube"]["status"], "in_stock");

    fx.cleanup().await;
}

#[tokio::test]
async fn cancel_and_mark_leave_a_consistent_final_state() {
    let fx = fixture("cancel-race").await;
    fx.seed_tube("R-C-1").await;
    let created = fx
        .as_operator(post_json(
            "/api/checks",
            serde_json::json!({"location": "A1"}),
        ))
        .await;
    let check_id = created.json()["id"].as_i64().unwrap();

    let pool = fx.pool.clone();
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let task = {
        let barrier = barrier.clone();
        tokio::spawn(async move {
            let service = steelpipe_service::CheckService::new(pool);
            let actor = Identity {
                user_id: 2,
                username: "op".into(),
                display_name: "Op".into(),
                role: Role::Operator,
                token_version: 0,
            };
            barrier.wait().await;
            service
                .mark_item(
                    &actor,
                    check_id,
                    "R-C-1",
                    steelpipe_domain::CheckResult::Missing,
                    None,
                )
                .await
        })
    };
    barrier.wait().await;
    let cancel = fx
        .as_operator(post_json(
            &format!("/api/checks/{check_id}/cancel"),
            serde_json::json!({}),
        ))
        .await;
    let mark = task.await.unwrap();
    assert_ne!(cancel.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(
        (cancel.status == StatusCode::NO_CONTENT && mark.is_ok())
            || (mark.is_err() && cancel.status != StatusCode::INTERNAL_SERVER_ERROR),
        "合法顺序是先核对后取消，或先取消后拒绝核对：cancel={} mark={mark:?}",
        cancel.status
    );
    let detail = fx
        .as_operator(get(&format!("/api/checks/{check_id}")))
        .await;
    if detail.json()["session"]["status"] == "cancelled" {
        assert_eq!(
            detail.json()["items"].as_array().unwrap().len(),
            usize::from(mark.is_ok()),
            "先取消后不得写入；先核对后取消保留历史明细"
        );
    }
    fx.cleanup().await;
}

#[tokio::test]
async fn cancel_missing_check_is_404() {
    let fx = fixture("cancel-missing").await;
    let reply = fx
        .as_operator(post_json(
            "/api/checks/999999/cancel",
            serde_json::json!({}),
        ))
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND, "body: {}", reply.body);
    fx.cleanup().await;
}

#[tokio::test]
async fn cancelled_check_leaves_stock_untouched() {
    let fx = fixture("cancel").await;
    fx.seed_tube("P-3").await;
    let created = fx
        .as_operator(post_json(
            "/api/checks",
            serde_json::json!({"location": "A1"}),
        ))
        .await;
    let check_id = created.json()["id"].as_i64().unwrap();
    fx.as_operator(post_json(
        &format!("/api/checks/{check_id}/items"),
        serde_json::json!({"pipe_id": "P-3", "result": "missing"}),
    ))
    .await;

    let cancelled = fx
        .as_operator(post_json(
            &format!("/api/checks/{check_id}/cancel"),
            serde_json::json!({}),
        ))
        .await;
    assert_eq!(cancelled.status, StatusCode::NO_CONTENT);

    let tube = fx.as_operator(get("/api/tubes/P-3")).await;
    assert_eq!(tube.json()["tube"]["status"], "in_stock");

    // 已取消的盘点单不能再确认
    let confirm = fx
        .as_admin(post_json(
            &format!("/api/checks/{check_id}/confirm"),
            serde_json::json!({}),
        ))
        .await;
    assert_eq!(confirm.status, StatusCode::CONFLICT);

    fx.cleanup().await;
}
