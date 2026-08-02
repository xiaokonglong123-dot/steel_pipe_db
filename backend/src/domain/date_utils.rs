//! Shared date/time parsing helpers for the repository layer.
//!
//! The DTO layer intentionally keeps date fields as `Option<String>` (JSON
//! interaction), while DB columns are `TIMESTAMPTZ`. These helpers bridge the
//! two: repos parse the DTO string into a `DateTime<Utc>` before binding so
//! PostgreSQL accepts the value.

use chrono::{DateTime, NaiveDate, Utc};

/// Parse a date string into a UTC timestamp.
///
/// Accepts either a bare calendar date (`2025-06-01`) or a full RFC 3339 /
/// ISO 8601 timestamp (`2025-06-01T10:30:00Z`). Bare dates are treated as
/// midnight UTC. Returns `None` for any unparseable input — callers decide
/// whether that means "skip the field" or "reject the request".
pub fn parse_date(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .ok()
        .map(|d| d.and_hms_opt(0, 0, 0).expect("midnight is a valid time").and_utc())
}

/// Parse an optional date string into an optional UTC timestamp.
///
/// `None` in, `None` out. `Some(unparseable)` also yields `None`, matching
/// the "skip this field" semantics used by dynamic UPDATE builders.
pub fn parse_opt_date(s: Option<&str>) -> Option<DateTime<Utc>> {
    s.and_then(parse_date)
}
