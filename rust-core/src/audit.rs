//! Audit submodule — hash-chain computation and verification (pure functions).
//!
//! # Hashing
//! Each log entry hash is `sha256(prev_hash ‖ action ‖ detail ‖ timestamp)`.
//! The genesis hash is `sha256("AYANAMI_REI_GENESIS")` (computed by Python).
//!
//! # Design
//! All functions are stateless.  Chain tip is stored in the DB and passed in
//! by Python; Rust never holds state.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Deterministic hash of one audit-log entry.
fn compute_hash_raw(prev_hash: &str, action: &str, detail: &str, timestamp: &str) -> String {
    let mut hasher = Sha256::default();
    hasher.update(prev_hash.as_bytes());
    hasher.update(action.as_bytes());
    hasher.update(detail.as_bytes());
    hasher.update(timestamp.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Extract a required string field from a dict-like Python object.
fn get_str(log: &Bound<'_, PyAny>, key: &str) -> PyResult<String> {
    let item = log.get_item(key).map_err(|_| {
        PyValueError::new_err(format!("missing required key: '{}'", key))
    })?;
    item.extract::<String>().map_err(|e| {
        PyValueError::new_err(format!("field '{}' is not a string: {}", key, e))
    })
}

// ---------------------------------------------------------------------------
// Public API (pyfunctions)
// ---------------------------------------------------------------------------

/// Compute the hash for one audit-log entry.
///
/// Hash = `sha256(prev_hash ‖ action ‖ detail ‖ timestamp)` as a hex string.
#[pyfunction]
fn compute_hash(prev_hash: &str, action: &str, detail: &str, timestamp: &str) -> String {
    compute_hash_raw(prev_hash, action, detail, timestamp)
}

/// Verify a list of audit-log dicts (full-table scan, best effort).
///
/// Each dict must contain `prev_hash`, `action`, `detail`, `timestamp`, and
/// `current_hash`.  Optionally `id` for identifying entries in the result.
///
/// Returns a list of dicts for entries whose recomputed hash does **not**
/// match the stored `current_hash`.  An empty list means the chain is intact.
#[pyfunction]
fn verify_chain(py: Python, logs: Vec<PyObject>) -> PyResult<Vec<PyObject>> {
    let mut failures: Vec<PyObject> = Vec::new();

    for (i, log_obj) in logs.iter().enumerate() {
        let log = log_obj.bind(py);

        let prev_hash = get_str(log, "prev_hash")?;
        let action = get_str(log, "action")?;
        let detail = get_str(log, "detail")?;
        let timestamp = get_str(log, "timestamp")?;
        let current_hash = get_str(log, "current_hash")?;

        let expected = compute_hash_raw(&prev_hash, &action, &detail, &timestamp);

        if expected != current_hash {
            let dict = PyDict::new(py);
            let log_id: i64 = log
                .get_item("id")
                .ok()
                .and_then(|v| v.extract::<i64>().ok())
                .unwrap_or(i as i64);
            dict.set_item("id", log_id)?;
            dict.set_item("ok", false)?;
            dict.set_item("expected", expected)?;
            dict.set_item("actual", current_hash)?;
            failures.push(dict.into());
        }
    }

    Ok(failures)
}

/// Verify a contiguous chunk of audit-log entries (batched / incremental).
///
/// # Constraints
/// - `logs.len()` must be ≤ 20 000 (otherwise `ValueError` is raised).
/// - The first log's `prev_hash` must equal `expected_prev_hash`.
/// - Each subsequent log's `prev_hash` must equal the previous log's computed
///   `current_hash` (chain-continuity check).
///
/// # Returns
/// A dict: `{"ok": bool, "broken_index": int | None, "tip": str}`.
/// `tip` is the last successfully computed `current_hash` (or `expected_prev_hash`
/// if the chunk was empty).
#[pyfunction]
fn verify_chain_chunk(
    py: Python,
    _start_id: i64,
    expected_prev_hash: &str,
    logs: Vec<PyObject>,
) -> PyResult<PyObject> {
    if logs.len() > 20_000 {
        return Err(PyValueError::new_err(
            format!(
                "chunk size {} exceeds maximum of 20 000 entries",
                logs.len()
            ),
        ));
    }

    let mut prev = expected_prev_hash.to_string();
    let mut broken_index: Option<i64> = None;
    let mut tip = prev.clone();

    for (i, log_obj) in logs.iter().enumerate() {
        let log = log_obj.bind(py);

        let log_prev_hash = get_str(log, "prev_hash")?;
        let action = get_str(log, "action")?;
        let detail = get_str(log, "detail")?;
        let timestamp = get_str(log, "timestamp")?;

        // Chain-continuity: this entry's prev_hash must match the previous
        // entry's computed hash (or expected_prev_hash for the first entry).
        if log_prev_hash != prev {
            broken_index = Some(i as i64);
            break;
        }

        let computed = compute_hash_raw(&prev, &action, &detail, &timestamp);
        let stored = get_str(log, "current_hash")?;

        if computed != stored {
            broken_index = Some(i as i64);
            break;
        }

        prev = computed;
        tip = prev.clone();
    }

    let result = PyDict::new(py);
    result.set_item("ok", broken_index.is_none())?;
    result.set_item("broken_index", broken_index)?;
    result.set_item("tip", tip)?;

    Ok(result.into())
}

// ---------------------------------------------------------------------------
// Submodule registration
// ---------------------------------------------------------------------------
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute_hash, m)?)?;
    m.add_function(wrap_pyfunction!(verify_chain, m)?)?;
    m.add_function(wrap_pyfunction!(verify_chain_chunk, m)?)?;
    Ok(())
}
