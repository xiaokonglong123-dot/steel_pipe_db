use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Extension,
};

/// A simple per-key rate limiter backed by an in-memory HashMap.
///
/// Each rate limit check uses a string key (typically `"{endpoint_group}:{client_ip}"`)
/// and a per-call max_requests + window duration so that the same shared instance
/// can enforce different policies for different route groups.
#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<Mutex<RateLimiterState>>,
}

struct RateLimiterState {
    buckets: HashMap<String, Vec<Instant>>,
}

impl RateLimiter {
    /// Create a new empty rate limiter.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RateLimiterState {
                buckets: HashMap::new(),
            })),
        }
    }

    /// Check whether a request identified by `key` should be allowed.
    ///
    /// * `key` — unique identifier for the rate limit bucket (e.g. `"login:127.0.0.1"`)
    /// * `max_requests` — maximum number of requests allowed within the window
    /// * `window_secs` — time window in seconds
    ///
    /// Returns `true` if the request is within limits, `false` if rate-limited.
    /// Also performs periodic cleanup of stale buckets to prevent memory leaks.
    pub fn check(&self, key: &str, max_requests: u64, window_secs: u64) -> bool {
        // Handle poisoned mutex gracefully — don't crash the server
        let mut state = match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::error!("Rate limiter mutex poisoned, resetting");
                poisoned.into_inner()
            }
        };
        let now = Instant::now();
        let window = Duration::from_secs(window_secs);

        // Periodic cleanup: remove stale buckets to prevent unbounded memory growth.
        // This runs on every check call; the cost is amortized across all requests.
        state.buckets.retain(|_, timestamps| {
            timestamps.retain(|t| now.duration_since(*t) < window);
            !timestamps.is_empty()
        });

        let timestamps = state.buckets.entry(key.to_string()).or_default();

        // Remove expired timestamps for this specific key
        timestamps.retain(|t| now.duration_since(*t) < window);

        if timestamps.len() as u64 >= max_requests {
            false
        } else {
            timestamps.push(now);
            true
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract client IP from the request, checking common proxy headers first.
fn client_ip(request: &Request) -> String {
    request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim())
        .or_else(|| {
            request
                .headers()
                .get("X-Real-IP")
                .and_then(|v| v.to_str().ok())
        })
        .unwrap_or("unknown")
        .to_string()
}

fn too_many_requests() -> Response {
    (StatusCode::TOO_MANY_REQUESTS, "429 Too Many Requests").into_response()
}

/// Rate limit for login/refresh endpoints — 5 requests per minute per IP.
pub async fn rate_limit_login(
    Extension(limiter): Extension<RateLimiter>,
    request: Request,
    next: Next,
) -> Response {
    let ip = client_ip(&request);
    if limiter.check(&format!("login:{}", ip), 5, 60) {
        next.run(request).await
    } else {
        too_many_requests()
    }
}

/// Rate limit for change-password endpoint — 3 requests per minute per IP.
pub async fn rate_limit_password_change(
    Extension(limiter): Extension<RateLimiter>,
    request: Request,
    next: Next,
) -> Response {
    let ip = client_ip(&request);
    if limiter.check(&format!("password:{}", ip), 3, 60) {
        next.run(request).await
    } else {
        too_many_requests()
    }
}

/// Rate limit for data-import endpoints — 10 requests per minute per IP.
pub async fn rate_limit_import(
    Extension(limiter): Extension<RateLimiter>,
    request: Request,
    next: Next,
) -> Response {
    let ip = client_ip(&request);
    if limiter.check(&format!("import:{}", ip), 10, 60) {
        next.run(request).await
    } else {
        too_many_requests()
    }
}
