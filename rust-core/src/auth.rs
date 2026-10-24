//! Authentication submodule — password hashing (argon2id) + JWT (HS256) + token validation.
//!
//! # Stateless (except JWT secret)
//! The only mutable state is the JWT secret set once at startup via `init()`.
//! Token versioning for revocation is passed in by Python (the caller reads
//! `users.token_version` from the DB), so Rust remains pure w.r.t. session state.

use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Secret key — set once via init()
// ---------------------------------------------------------------------------
static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();

fn secret() -> Option<&'static Vec<u8>> {
    JWT_SECRET.get()
}

// ---------------------------------------------------------------------------
// JWT claims (shared by access and refresh tokens)
// ---------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // user_id as string (JWT std)
    #[serde(default)]
    role: String,
    token_version: i64,
    #[serde(default)]
    jti: String,
    exp: usize,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
fn now_plus_secs(secs: u64) -> usize {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    (now + secs) as usize
}

// ---------------------------------------------------------------------------
// Public API (pyfunctions)
// ---------------------------------------------------------------------------

/// Initialise the JWT signing secret.  Must be called once at startup before
/// any token operations.  Idempotent — subsequent calls are silently ignored.
#[pyfunction]
fn init(secret_key: &str) {
    JWT_SECRET.get_or_init(|| secret_key.as_bytes().to_vec());
}

/// Hash a password with argon2id (default params: m=19456KiB, t=2, p=1).
/// Returns the PHC-encoded hash string.
#[pyfunction]
fn hash_password(password: &str) -> PyResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| PyValueError::new_err(format!("Hashing failed: {e}")))?;
    Ok(hash.to_string())
}

/// Verify a password against a PHC-encoded argon2id hash.
/// Returns `false` on any error (wrong password, malformed hash) — never panics.
#[pyfunction]
fn verify_password(password: &str, hash: &str) -> bool {
    let parsed = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

/// Issue an HS256 access token (valid 15 minutes).
///
/// Payload: `sub` (user_id), `role`, `token_version`, `exp`.
#[pyfunction]
fn issue_access_token(user_id: i64, role: &str, token_version: i64) -> PyResult<String> {
    let key = secret()
        .ok_or_else(|| PyRuntimeError::new_err("init() not called"))?;
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        token_version,
        jti: String::new(),
        exp: now_plus_secs(15 * 60),
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(key),
    )
    .map_err(|e| PyRuntimeError::new_err(format!("Token signing failed: {e}")))
}

/// Issue an HS256 refresh token (valid 30 days).
///
/// Payload: `sub`, `token_version`, `jti` (UUID v4), `exp`.
#[pyfunction]
fn issue_refresh_token(user_id: i64, token_version: i64) -> PyResult<String> {
    let key = secret()
        .ok_or_else(|| PyRuntimeError::new_err("init() not called"))?;
    let claims = Claims {
        sub: user_id.to_string(),
        role: String::new(),
        token_version,
        jti: Uuid::new_v4().to_string(),
        exp: now_plus_secs(30 * 24 * 60 * 60),
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(key),
    )
    .map_err(|e| PyRuntimeError::new_err(format!("Token signing failed: {e}")))
}

/// Decode and verify an HS256 token without token_version comparison.
///
/// Returns `None` if signature is invalid or token is expired.
/// Returns a dict on success with keys:
///   `user_id` (int), `role` (str, empty for refresh tokens),
///   `token_version` (int), `jti` (str, only present for refresh tokens).
///
/// Unlike `verify_token`, this does NOT compare `token_version` — the caller
/// (Python) is responsible for checking the version against the DB value.
#[pyfunction]
fn decode_claims(py: Python, token: &str) -> PyResult<Option<PyObject>> {
    let key = match secret() {
        Some(k) => k,
        None => return Ok(None),
    };

    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(key),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(d) => d,
        Err(_) => return Ok(None),
    };

    let claims = token_data.claims;
    let dict = PyDict::new(py);
    dict.set_item("user_id", claims.sub.parse::<i64>().unwrap_or(0))?;
    dict.set_item("role", claims.role)?;
    dict.set_item("token_version", claims.token_version)?;
    if !claims.jti.is_empty() {
        dict.set_item("jti", claims.jti)?;
    }
    Ok(Some(dict.into()))
}

/// Verify and decode an HS256 token.
///
/// Returns `None` if:
///   - signature is invalid / token is expired
///   - `token_version` in the claims does not match `current_token_version`
///
/// Returns a dict on success with keys:
///   `user_id` (int), `role` (str, empty for refresh tokens),
///   `token_version` (int), `jti` (str, only present for refresh tokens).
#[pyfunction]
fn verify_token(
    py: Python,
    token: &str,
    current_token_version: i64,
) -> PyResult<Option<PyObject>> {
    let key = match secret() {
        Some(k) => k,
        None => return Ok(None),
    };

    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(key),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(d) => d,
        Err(_) => return Ok(None),
    };

    let claims = token_data.claims;

    if claims.token_version != current_token_version {
        return Ok(None);
    }

    let dict = PyDict::new(py);
    dict.set_item("user_id", claims.sub.parse::<i64>().unwrap_or(0))?;
    dict.set_item("role", claims.role)?;
    dict.set_item("token_version", claims.token_version)?;
    if !claims.jti.is_empty() {
        dict.set_item("jti", claims.jti)?;
    }
    Ok(Some(dict.into()))
}

// ---------------------------------------------------------------------------
// Submodule registration
// ---------------------------------------------------------------------------
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init, m)?)?;
    m.add_function(wrap_pyfunction!(hash_password, m)?)?;
    m.add_function(wrap_pyfunction!(verify_password, m)?)?;
    m.add_function(wrap_pyfunction!(issue_access_token, m)?)?;
    m.add_function(wrap_pyfunction!(issue_refresh_token, m)?)?;
    m.add_function(wrap_pyfunction!(decode_claims, m)?)?;
    m.add_function(wrap_pyfunction!(verify_token, m)?)?;
    Ok(())
}
