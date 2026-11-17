//! 时间戳格式：RFC3339、微秒、`+00:00` 后缀。
//!
//! 审计哈希把 timestamp 字符串算进链，所以格式必须唯一确定。与旧实现的
//! `datetime.now(UTC).isoformat()` 逐字符一致，例如
//! `2026-11-17T10:35:38.105239+00:00`。

use chrono::{SecondsFormat, Utc};

/// 当前时刻的审计/GRE 时间戳。
pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Micros, false)
}

/// 把「若干秒之前」格式化成同一格式（限流窗口用）。
pub fn secs_ago(seconds: i64) -> String {
    (Utc::now() - chrono::Duration::seconds(seconds)).to_rfc3339_opts(SecondsFormat::Micros, false)
}

/// 把「若干秒之后」格式化成同一格式（锁定截止时间用）。
pub fn secs_later(seconds: i64) -> String {
    (Utc::now() + chrono::Duration::seconds(seconds)).to_rfc3339_opts(SecondsFormat::Micros, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_matches_python_isoformat() {
        let stamp = now_rfc3339();
        assert_eq!(stamp.len(), 32, "实际：{stamp}");
        assert!(stamp.ends_with("+00:00"), "实际：{stamp}");
        assert_eq!(&stamp[4..5], "-");
        assert_eq!(&stamp[10..11], "T");
        assert_eq!(&stamp[19..20], ".");
        // 微秒恰好 6 位
        assert!(stamp[20..26].chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn timestamps_are_lexicographically_ordered() {
        let older = secs_ago(60);
        let now = now_rfc3339();
        let later = secs_later(60);
        assert!(older < now && now < later, "{older} / {now} / {later}");
    }
}
