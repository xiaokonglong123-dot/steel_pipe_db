//! 会话 cookie、认证提取器、CSRF double-submit（A10–A12）。

use axum::extract::FromRequestParts;
use axum::http::header::{self, HeaderMap, HeaderValue};
use axum::http::request::Parts;
use steelpipe_service::{Identity, Session};

use crate::error::{ApiError, ApiReject, reject};
use crate::state::AppState;

/// cookie 与 header 名称集中定义。
pub mod consts {
    pub const ACCESS_COOKIE: &str = "access_token";
    pub const REFRESH_COOKIE: &str = "refresh_token";
    pub const CSRF_COOKIE: &str = "csrf_token";
    pub const CSRF_HEADER: &str = "x-csrf-token";
}

use consts::*;

/// 从 Cookie 头里取一个值（不依赖 cookie crate：格式就是 `k=v; k=v`）。
pub fn cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value)
}

/// 认证后的请求身份。
#[derive(Debug, Clone)]
pub struct AuthUser(pub Identity);

impl AuthUser {
    /// handler 里已经持有 headers 时用它，避免再写一遍 extractor 签名。
    pub async fn from_headers(headers: &HeaderMap, state: &AppState) -> Result<Self, ApiError> {
        let token = cookie(headers, ACCESS_COOKIE).ok_or(ApiError::Unauthorized)?;
        let identity = state.auth.authenticate(token).await?;
        Ok(AuthUser(identity))
    }

    pub fn require_admin(&self) -> Result<(), ApiError> {
        if self.0.role.is_admin() {
            Ok(())
        } else {
            Err(ApiError::Forbidden)
        }
    }

    pub fn require_operator(&self) -> Result<(), ApiError> {
        if self.0.role.can_write() {
            Ok(())
        } else {
            Err(ApiError::Forbidden)
        }
    }
}

impl FromRequestParts<AppState> for AuthUser {
    // 拒绝体必须带着请求头渲染，才能实现 401/302 双轨（A11）
    type Rejection = ApiReject;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = cookie(&parts.headers, ACCESS_COOKIE)
            .ok_or_else(|| reject(ApiError::Unauthorized, &parts.headers))?;
        let identity = state
            .auth
            .authenticate(token)
            .await
            .map_err(|error| reject(error.into(), &parts.headers))?;
        Ok(AuthUser(identity))
    }
}

/// CSRF double-submit 校验：cookie 值必须等于 `X-CSRF-Token` 头。
#[derive(Debug)]
pub struct CsrfOk;

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}

/// CSRF 校验的单一实现：cookie 必须存在，且与 header 常量时间相等。
pub fn check_csrf(headers: &HeaderMap) -> Result<(), ApiError> {
    let expected = cookie(headers, CSRF_COOKIE).ok_or(ApiError::Forbidden)?;
    let presented = headers
        .get(CSRF_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::Forbidden)?;
    if constant_time_eq(expected.as_bytes(), presented.as_bytes()) {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

impl FromRequestParts<AppState> for CsrfOk {
    type Rejection = ApiReject;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        check_csrf(&parts.headers).map_err(|error| reject(error, &parts.headers))?;
        Ok(CsrfOk)
    }
}

fn cookie_header(name: &str, value: &str, http_only: bool, secure: bool) -> HeaderValue {
    let mut cookie = format!("{name}={value}; Path=/; SameSite=Lax");
    if http_only {
        cookie.push_str("; HttpOnly");
    }
    if secure {
        cookie.push_str("; Secure");
    }
    HeaderValue::from_str(&cookie).expect("cookie 头不合法")
}

/// 登录/刷新成功后下发三个 cookie（A10）。
pub fn session_headers(session: &Session, secure: bool) -> HeaderMap {
    let mut headers = HeaderMap::new();
    // 无 Max-Age：浏览器会话级，有效期靠 JWT exp（与旧设计一致）
    headers.append(
        header::SET_COOKIE,
        cookie_header(ACCESS_COOKIE, &session.access_token, true, secure),
    );
    headers.append(
        header::SET_COOKIE,
        cookie_header(REFRESH_COOKIE, &session.refresh_token, true, secure),
    );
    headers.append(
        header::SET_COOKIE,
        cookie_header(CSRF_COOKIE, &session.csrf_token, false, secure),
    );
    headers
}

/// 登出/失效时清 cookie。
pub fn clear_session_headers(secure: bool) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, http_only) in [
        (ACCESS_COOKIE, true),
        (REFRESH_COOKIE, true),
        (CSRF_COOKIE, false),
    ] {
        let mut value = format!("{name}=; Path=/; SameSite=Lax; Max-Age=0");
        if http_only {
            value.push_str("; HttpOnly");
        }
        if secure {
            value.push_str("; Secure");
        }
        headers.append(
            header::SET_COOKIE,
            HeaderValue::from_str(&value).expect("cookie 头不合法"),
        );
    }
    headers
}

/// `GET /api/auth/csrf` 用：只下发 CSRF cookie。
pub fn csrf_headers(token: &str, secure: bool) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        cookie_header(CSRF_COOKIE, token, false, secure),
    );
    headers
}

/// 随机 CSRF token（32 字节 hex）。登录前也能拿到，避免首屏空手。
pub fn new_csrf_token() -> String {
    use rand_core::{OsRng, RngCore};
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// 取客户端 IP：优先 `X-Forwarded-For` 第一段，否则用连接信息。
pub fn client_ip(headers: &HeaderMap, peer: Option<std::net::SocketAddr>) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .or_else(|| peer.map(|addr| addr.ip().to_string()))
        .unwrap_or_else(|| "unknown".to_owned())
}
