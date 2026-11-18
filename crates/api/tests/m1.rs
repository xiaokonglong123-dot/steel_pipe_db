//! M1 端到端：认证、CSRF、401/302 分流、限流、用户管理（A10–A21、B 域矩阵）。
//!
//! 走真实 axum Router + 真实 SQLite 临时库，只把时钟与网络替换成测试装置。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;

use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, Response, StatusCode, header};
use http_body_util::BodyExt;
use steelpipe_api::{AppState, router};
use steelpipe_auth::{Keys, hash_password};
use steelpipe_domain::Role;
use steelpipe_infra::{DbPool, repo::users, time, uow::Uow};
use steelpipe_service::{AuthService, UserService};
use tower::ServiceExt;

struct Harness {
    app: Router,
    pool: DbPool,
    path: PathBuf,
}

impl Harness {
    async fn new(tag: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("steelpipe-api-{tag}-{nanos}.db"));
        let pool = steelpipe_infra::connect(&format!("sqlite://{}", path.display()), 8)
            .await
            .expect("打开临时库失败");
        steelpipe_infra::migrate(&pool).await.expect("迁移失败");

        let keys = Keys::from_secret(b"api-test-secret-at-least-32-bytes!!");
        let auth = std::sync::Arc::new(AuthService::new(pool.clone(), keys));
        let users = std::sync::Arc::new(UserService::new(pool.clone()));
        let state = AppState::new(auth, users, None, false, "test");
        Self {
            app: router(state),
            pool,
            path,
        }
    }

    async fn seed(&self, username: &str, password: &str, role: Role) {
        let hash = hash_password(password).expect("哈希失败");
        let mut uow = Uow::begin(&self.pool).await.unwrap();
        users::insert(
            uow.conn(),
            username,
            &hash,
            username,
            role,
            &time::now_rfc3339(),
        )
        .await
        .unwrap();
        uow.commit().await.unwrap();
    }

    async fn send(&self, request: Request<Body>) -> Reply {
        let request = with_peer(request);
        let response = self.app.clone().oneshot(request).await.expect("请求失败");
        Reply::from_response(response).await
    }

    async fn cleanup(self) {
        self.pool.close().await;
        let _ = std::fs::remove_file(&self.path);
    }
}

fn with_peer(mut request: Request<Body>) -> Request<Body> {
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 40000))));
    request
}

struct Reply {
    status: StatusCode,
    headers: axum::http::HeaderMap,
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
        Self {
            status,
            headers,
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

fn get_html(path: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header(header::ACCEPT, "text/html,application/xhtml+xml")
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

/// 领取 CSRF token（模拟前端首屏动作）。
async fn csrf(harness: &Harness) -> (String, String) {
    let reply = harness.send(get("/api/auth/csrf")).await;
    assert_eq!(reply.status, StatusCode::OK);
    let token = reply.json()["csrf_token"].as_str().unwrap().to_owned();
    (token, reply.cookie_header(&["csrf_token"]))
}

#[tokio::test]
async fn login_requires_csrf() {
    let fx = Harness::new("csrf").await;
    fx.seed("alice", "alice-pw-1", Role::Admin).await;

    let reply = fx
        .send(post_json(
            "/api/auth/login",
            serde_json::json!({"username": "alice", "password": "alice-pw-1"}),
        ))
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);

    // cookie 有但 header 不匹配 → 仍然 403
    let (_, cookie) = csrf(&fx).await;
    let bad = with_csrf(
        with_cookies(
            post_json(
                "/api/auth/login",
                serde_json::json!({"username": "alice", "password": "alice-pw-1"}),
            ),
            &cookie,
        ),
        "deadbeef",
    );
    assert_eq!(fx.send(bad).await.status, StatusCode::FORBIDDEN);

    fx.cleanup().await;
}

#[tokio::test]
async fn login_success_sets_cookies_and_me_returns_identity() {
    let fx = Harness::new("login").await;
    fx.seed("alice", "alice-pw-1", Role::Operator).await;

    let (token, csrf_cookie) = csrf(&fx).await;
    let request = with_csrf(
        with_cookies(
            post_json(
                "/api/auth/login",
                serde_json::json!({"username": "alice", "password": "alice-pw-1"}),
            ),
            &csrf_cookie,
        ),
        &token,
    );
    let reply = fx.send(request).await;
    assert_eq!(reply.status, StatusCode::OK, "body: {}", reply.body);
    assert_eq!(reply.json()["user"]["role"], "operator");
    assert!(reply.cookies.contains_key("access_token"));
    assert!(reply.cookies.contains_key("refresh_token"));
    assert!(reply.cookies.contains_key("csrf_token"));

    // Set-Cookie 必须是 httpOnly；会话级（无 Max-Age）
    let raw = reply
        .headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(raw.contains("HttpOnly"), "实际：{raw}");
    assert!(
        !raw.contains("Max-Age=3"),
        "登录 cookie 不应带长 Max-Age：{raw}"
    );

    let me = fx
        .send(with_cookies(
            get("/api/auth/me"),
            &reply.cookie_header(&["access_token"]),
        ))
        .await;
    assert_eq!(me.status, StatusCode::OK);
    assert_eq!(me.json()["username"], "alice");

    fx.cleanup().await;
}

#[tokio::test]
async fn unauthenticated_requests_are_401_json_or_303_to_login() {
    let fx = Harness::new("dual").await;

    let api = fx.send(get("/api/auth/me")).await;
    assert_eq!(api.status, StatusCode::UNAUTHORIZED);
    assert_eq!(api.json()["status"], 401);
    assert_eq!(api.json()["title"], "unauthorized");
    assert!(api.headers.contains_key(header::WWW_AUTHENTICATE));

    let browser = fx.send(get_html("/api/auth/me")).await;
    assert_eq!(browser.status, StatusCode::SEE_OTHER);
    assert_eq!(browser.headers[header::LOCATION], "/login");

    fx.cleanup().await;
}

#[tokio::test]
async fn refresh_rotates_and_rejects_stale_token() {
    let fx = Harness::new("refresh").await;
    fx.seed("bob", "bob-pw-123", Role::Operator).await;

    let (token, csrf_cookie) = csrf(&fx).await;
    let login = fx
        .send(with_csrf(
            with_cookies(
                post_json(
                    "/api/auth/login",
                    serde_json::json!({"username": "bob", "password": "bob-pw-123"}),
                ),
                &csrf_cookie,
            ),
            &token,
        ))
        .await;
    assert_eq!(login.status, StatusCode::OK);
    let old_refresh = login.cookie_header(&["refresh_token", "csrf_token"]);
    let old_csrf = login.cookies["csrf_token"].clone();

    let rotated = fx
        .send(with_csrf(
            with_cookies(
                post_json("/api/auth/refresh", serde_json::json!({})),
                &old_refresh,
            ),
            &old_csrf,
        ))
        .await;
    assert_eq!(rotated.status, StatusCode::OK, "body: {}", rotated.body);
    let new_refresh = rotated.cookie_header(&["refresh_token", "csrf_token"]);
    assert_ne!(old_refresh, new_refresh);

    // 旧 refresh 已被轮换 → 401
    let stale = fx
        .send(with_csrf(
            with_cookies(
                post_json("/api/auth/refresh", serde_json::json!({})),
                &old_refresh,
            ),
            &old_csrf,
        ))
        .await;
    assert_eq!(stale.status, StatusCode::UNAUTHORIZED);

    fx.cleanup().await;
}

#[tokio::test]
async fn logout_kills_refresh_and_requires_csrf() {
    let fx = Harness::new("logout").await;
    fx.seed("carol", "carol-pw-1", Role::Operator).await;

    let (token, csrf_cookie) = csrf(&fx).await;
    let login = fx
        .send(with_csrf(
            with_cookies(
                post_json(
                    "/api/auth/login",
                    serde_json::json!({"username": "carol", "password": "carol-pw-1"}),
                ),
                &csrf_cookie,
            ),
            &token,
        ))
        .await;
    let auth_cookie = login.cookie_header(&["access_token", "refresh_token", "csrf_token"]);
    let csrf_token = login.cookies["csrf_token"].clone();

    // 无 CSRF → 403
    let no_csrf = fx
        .send(with_cookies(
            post_json("/api/auth/logout", serde_json::json!({})),
            &auth_cookie,
        ))
        .await;
    assert_eq!(no_csrf.status, StatusCode::FORBIDDEN);

    let logout = fx
        .send(with_csrf(
            with_cookies(
                post_json("/api/auth/logout", serde_json::json!({})),
                &auth_cookie,
            ),
            &csrf_token,
        ))
        .await;
    assert_eq!(logout.status, StatusCode::OK);
    assert_eq!(logout.cookies["access_token"], "");

    // 登出后 refresh 失效
    let refresh = fx
        .send(with_csrf(
            with_cookies(
                post_json("/api/auth/refresh", serde_json::json!({})),
                &login.cookie_header(&["refresh_token", "csrf_token"]),
            ),
            &csrf_token,
        ))
        .await;
    assert_eq!(refresh.status, StatusCode::UNAUTHORIZED);

    fx.cleanup().await;
}

#[tokio::test]
async fn users_endpoints_require_admin() {
    let fx = Harness::new("rbac").await;
    fx.seed("root", "root-pw-123", Role::Admin).await;
    fx.seed("op", "op-pw-1234", Role::Operator).await;
    fx.seed("viewer", "viewer-pw-1", Role::Viewer).await;

    let login = |who: &'static str, pw: &'static str| {
        let fx = &fx;
        async move {
            let (token, csrf_cookie) = csrf(fx).await;
            let reply = fx
                .send(with_csrf(
                    with_cookies(
                        post_json(
                            "/api/auth/login",
                            serde_json::json!({"username": who, "password": pw}),
                        ),
                        &csrf_cookie,
                    ),
                    &token,
                ))
                .await;
            assert_eq!(
                reply.status,
                StatusCode::OK,
                "{who} 登录失败：{}",
                reply.body
            );
            reply
        }
    };

    let root = login("root", "root-pw-123").await;
    let root_cookies = root.cookie_header(&["access_token", "refresh_token", "csrf_token"]);
    let root_csrf = root.cookies["csrf_token"].clone();
    assert_eq!(
        fx.send(with_cookies(get("/api/users"), &root_cookies))
            .await
            .status,
        StatusCode::OK
    );

    let op = login("op", "op-pw-1234").await;
    let op_cookies = op.cookie_header(&["access_token", "refresh_token", "csrf_token"]);
    assert_eq!(
        fx.send(with_cookies(get("/api/users"), &op_cookies))
            .await
            .status,
        StatusCode::FORBIDDEN
    );

    // viewer 也不能写：先取 CSRF 再试
    let viewer = login("viewer", "viewer-pw-1").await;
    assert_eq!(
        fx.send(with_cookies(
            get("/api/users"),
            &viewer.cookie_header(&["access_token"])
        ))
        .await
        .status,
        StatusCode::FORBIDDEN
    );

    // admin 建号
    let created = fx
        .send(with_csrf(
            with_cookies(
                post_json(
                    "/api/users",
                    serde_json::json!({
                        "username": "newop",
                        "password": "newop-pw-1",
                        "display_name": "新操作员",
                        "role": "operator"
                    }),
                ),
                &root_cookies,
            ),
            &root_csrf,
        ))
        .await;
    assert_eq!(
        created.status,
        StatusCode::CREATED,
        "body: {}",
        created.body
    );
    assert_eq!(created.json()["username"], "newop");

    // 重复用户名 → 409
    let duplicate = fx
        .send(with_csrf(
            with_cookies(
                post_json(
                    "/api/users",
                    serde_json::json!({
                        "username": "newop",
                        "password": "newop-pw-1",
                        "display_name": "重复",
                        "role": "viewer"
                    }),
                ),
                &root_cookies,
            ),
            &root_csrf,
        ))
        .await;
    assert_eq!(duplicate.status, StatusCode::CONFLICT);

    fx.cleanup().await;
}

#[tokio::test]
async fn lockout_returns_429_with_retry_after() {
    let fx = Harness::new("lockout").await;
    fx.seed("dave", "dave-pw-123", Role::Operator).await;
    let (token, csrf_cookie) = csrf(&fx).await;

    for i in 0..10 {
        let reply = fx
            .send(with_csrf(
                with_cookies(
                    post_json(
                        "/api/auth/login",
                        serde_json::json!({"username": "dave", "password": "wrong"}),
                    ),
                    &csrf_cookie,
                ),
                &token,
            ))
            .await;
        // 前 5 次是 401，第 6 次起 IP 层先触发 429（同一 IP）
        assert!(
            reply.status == StatusCode::UNAUTHORIZED
                || reply.status == StatusCode::TOO_MANY_REQUESTS,
            "第 {} 次：{}",
            i + 1,
            reply.status
        );
    }

    let blocked = fx
        .send(with_csrf(
            with_cookies(
                post_json(
                    "/api/auth/login",
                    serde_json::json!({"username": "dave", "password": "dave-pw-123"}),
                ),
                &csrf_cookie,
            ),
            &token,
        ))
        .await;
    assert_eq!(blocked.status, StatusCode::TOO_MANY_REQUESTS);
    assert!(blocked.headers.contains_key(header::RETRY_AFTER));

    fx.cleanup().await;
}

#[tokio::test]
async fn untrusted_forwarded_header_cannot_bypass_ip_limit() {
    let fx = Harness::new("untrusted-forwarded").await;
    fx.seed("dave", "correct-pw-123", Role::Operator).await;
    let (token, csrf_cookie) = csrf(&fx).await;
    for i in 0..5 {
        let mut request = with_csrf(
            with_cookies(
                post_json(
                    "/api/auth/login",
                    serde_json::json!({"username": "dave", "password": "wrong"}),
                ),
                &csrf_cookie,
            ),
            &token,
        );
        request.headers_mut().insert(
            "x-forwarded-for",
            format!("198.51.100.{i}").parse().unwrap(),
        );
        assert_eq!(fx.send(request).await.status, StatusCode::UNAUTHORIZED);
    }
    let mut request = with_csrf(
        with_cookies(
            post_json(
                "/api/auth/login",
                serde_json::json!({"username": "dave", "password": "wrong"}),
            ),
            &csrf_cookie,
        ),
        &token,
    );
    request
        .headers_mut()
        .insert("x-forwarded-for", "198.51.100.200".parse().unwrap());
    assert_eq!(fx.send(request).await.status, StatusCode::TOO_MANY_REQUESTS);
    fx.cleanup().await;
}

#[tokio::test]
async fn audit_total_uses_the_action_filter() {
    let fx = Harness::new("audit-filter").await;
    fx.seed("root", "root-pw-123", Role::Admin).await;
    let (token, csrf_cookie) = csrf(&fx).await;
    let login = fx
        .send(with_csrf(
            with_cookies(
                post_json(
                    "/api/auth/login",
                    serde_json::json!({"username": "root", "password": "root-pw-123"}),
                ),
                &csrf_cookie,
            ),
            &token,
        ))
        .await;
    let cookies = login.cookie_header(&["access_token", "refresh_token", "csrf_token"]);
    let csrf = login.cookies["csrf_token"].clone();
    let created = fx
        .send(with_csrf(
            with_cookies(
                post_json(
                    "/api/locations",
                    serde_json::json!({"code": "AUDIT-FILTER"}),
                ),
                &cookies,
            ),
            &csrf,
        ))
        .await;
    assert_eq!(
        created.status,
        StatusCode::CREATED,
        "body: {}",
        created.body
    );
    let filtered = fx
        .send(with_cookies(
            get("/api/audit?action=create_location"),
            &cookies,
        ))
        .await;
    assert_eq!(filtered.status, StatusCode::OK);
    assert_eq!(filtered.json()["total"], 1);
    assert_eq!(filtered.json()["entries"].as_array().unwrap().len(), 1);
    fx.cleanup().await;
}

#[tokio::test]
async fn db_health_is_503_when_probe_missing() {
    let fx = Harness::new("probe").await;
    let reply = fx.send(get("/api/health/db")).await;
    assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(
        reply.json()["detail"]
            .as_str()
            .unwrap()
            .contains("STEELPIPE_DATABASE_URL")
    );
    fx.cleanup().await;
}
