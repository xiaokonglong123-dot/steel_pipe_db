//! 审计哈希链：纯函数，无 IO、无状态。
//!
//! 搬自旧 `rust-core/src/audit.rs`，链算法逐字节一致：
//!
//! ```text
//! current_hash = sha256(prev_hash ‖ action ‖ detail ‖ timestamp)   // 十六进制小写
//! ```
//!
//! 一致性由 `golden_vectors_match_legacy_implementation` 锁定：向量由旧 Python
//! 实现算出，改了算法这条测试必挂。
//!
//! `detail` 的规范化由 [`detail`] 负责：紧凑 JSON、字段顺序即结构体声明顺序。
//! 链尾（tip）不驻留内存，由调用方从 DB 读出后传入。

#![forbid(unsafe_code)]

use serde::Serialize;
use sha2::{Digest, Sha256};

/// 创世哈希：`sha256("AYANAMI_REI_GENESIS")`。
pub const GENESIS_HASH: &str = "2e3874e8f41f106cc35d41312497bf350acebadc6ab2ebf6728631a9c180cd40";

/// 单批校验上限（旧实现同样是 20 000，防大突发爆内存）。
pub const MAX_CHUNK: usize = 20_000;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VerifyError {
    #[error("批次过大：{len} > {max}")]
    ChunkTooLarge { len: usize, max: usize },
    #[error("第 {index} 条的 prev_hash 与上一条不衔接")]
    Discontinuous {
        index: usize,
        expected: String,
        actual: String,
    },
    #[error("第 {index} 条的哈希不匹配")]
    HashMismatch {
        index: usize,
        expected: String,
        actual: String,
    },
}

/// 链条目（只读视图，避免复制 20 万行字符串）。
#[derive(Debug, Clone)]
pub struct Entry<'a> {
    pub prev_hash: &'a str,
    pub action: &'a str,
    pub detail: &'a str,
    pub timestamp: &'a str,
    pub current_hash: &'a str,
}

/// 计算一条审计记录的哈希。
pub fn compute_hash(prev_hash: &str, action: &str, detail: &str, timestamp: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash.as_bytes());
    hasher.update(action.as_bytes());
    hasher.update(detail.as_bytes());
    hasher.update(timestamp.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// 分批校验一段连续记录。
///
/// - `expected_prev_hash`：本批第一条的 prev_hash 应当等于的值。
/// - 返回成功时的链尾（本批最后一条的 current_hash；空批则原样返回）。
pub fn verify_chunk(
    expected_prev_hash: &str,
    entries: &[Entry<'_>],
) -> Result<String, VerifyError> {
    if entries.len() > MAX_CHUNK {
        return Err(VerifyError::ChunkTooLarge {
            len: entries.len(),
            max: MAX_CHUNK,
        });
    }

    let mut prev = expected_prev_hash.to_owned();

    for (index, entry) in entries.iter().enumerate() {
        if entry.prev_hash != prev {
            return Err(VerifyError::Discontinuous {
                index,
                expected: prev,
                actual: entry.prev_hash.to_owned(),
            });
        }
        let computed = compute_hash(prev.as_str(), entry.action, entry.detail, entry.timestamp);
        if computed != entry.current_hash {
            return Err(VerifyError::HashMismatch {
                index,
                expected: computed,
                actual: entry.current_hash.to_owned(),
            });
        }
        prev = computed;
    }

    Ok(prev)
}

/// 规范化审计 detail：紧凑 JSON（无空格），字段顺序即结构体声明顺序。
///
/// 所有写审计的调用点必须经过这里，否则同一个业务对象可能产生两种字节序列，
/// 链仍然"能验证"，但同一条记录换台机器重放会算出不同哈希。
pub fn detail<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 向量由旧 Python 实现（hashlib.sha256(prev+action+detail+ts)）产出。
    #[test]
    fn golden_vectors_match_legacy_implementation() {
        let cases = [
            (
                GENESIS_HASH,
                "stock_in",
                r#"{"pipe_id":"P-0001","location":"A1"}"#,
                "2026-11-17T10:35:38.105239+00:00",
                "b72802806aa056c3ed30913413afd569f43bab3a224439bc3696351911f709bb",
            ),
            (
                GENESIS_HASH,
                "login",
                "{}",
                "2026-11-17T00:00:00.000000+00:00",
                "ab260437d1bb4b8ce4ac7ca47708f64facad514b057b0e08dec9bff0c0872533",
            ),
            (
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                "login_lockout",
                r#"{"username":"admin","attempts":10}"#,
                "2026-11-17T00:00:01.000000+00:00",
                "c9afc1991cc1035b9bad1c7801ae0e0650d26141a6abe053d78416c385d37a23",
            ),
        ];

        for (prev, action, detail_text, ts, expected) in cases {
            assert_eq!(
                compute_hash(prev, action, detail_text, ts),
                expected,
                "action={action}"
            );
        }
    }

    #[test]
    fn genesis_hash_is_sha256_of_the_reference_phrase() {
        let mut hasher = Sha256::new();
        hasher.update(b"AYANAMI_REI_GENESIS");
        assert_eq!(format!("{:x}", hasher.finalize()), GENESIS_HASH);
    }

    /// 构造一条完整链条（prev / action / detail / ts / hash 全部持有所有权）。
    struct Row {
        prev: String,
        action: String,
        detail: String,
        ts: String,
        hash: String,
    }

    fn chain(actions: &[&str], details: &[&str]) -> Vec<Row> {
        let mut prev = GENESIS_HASH.to_owned();
        let mut rows = Vec::new();
        for (i, (action, detail_text)) in actions.iter().zip(details).enumerate() {
            let ts = format!("2026-11-17T00:00:{i:02}.000000+00:00");
            let hash = compute_hash(&prev, action, detail_text, &ts);
            rows.push(Row {
                prev: prev.clone(),
                action: (*action).to_owned(),
                detail: (*detail_text).to_owned(),
                ts,
                hash: hash.clone(),
            });
            prev = hash;
        }
        rows
    }

    fn entry(row: &Row) -> Entry<'_> {
        Entry {
            prev_hash: &row.prev,
            action: &row.action,
            detail: &row.detail,
            timestamp: &row.ts,
            current_hash: &row.hash,
        }
    }

    #[test]
    fn verifies_a_clean_chain_and_reports_tip() {
        let rows = chain(&["in", "out", "move"], &["{}", r#"{"a":1}"#, "{}"]);
        let entries: Vec<Entry<'_>> = rows.iter().map(entry).collect();
        let tip = verify_chunk(GENESIS_HASH, &entries).expect("应通过");
        assert_eq!(tip, rows.last().unwrap().hash);
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn second_chunk_continues_from_first_tip() {
        let rows = chain(&["in", "out"], &["{}", "{}"]);
        let first: Vec<Entry<'_>> = rows[..1].iter().map(entry).collect();
        let second: Vec<Entry<'_>> = rows[1..].iter().map(entry).collect();

        let tip = verify_chunk(GENESIS_HASH, &first).expect("第一批应通过");
        let tip = verify_chunk(&tip, &second).expect("第二批应通过");
        assert_eq!(tip, rows.last().unwrap().hash);
    }

    #[test]
    fn detects_tampered_detail() {
        let rows = chain(&["in"], &["{}"]);
        let mut entries: Vec<Entry<'_>> = rows.iter().map(entry).collect();
        entries[0].detail = r#"{"tampered":true}"#;
        assert!(matches!(
            verify_chunk(GENESIS_HASH, &entries),
            Err(VerifyError::HashMismatch { index: 0, .. })
        ));
    }

    #[test]
    fn detects_discontinuity() {
        let rows = chain(&["in", "out"], &["{}", "{}"]);
        // 丢掉中间一条：第二条的 prev_hash 不再衔接
        let entries = [entry(&rows[1])];
        assert!(matches!(
            verify_chunk(GENESIS_HASH, &entries),
            Err(VerifyError::Discontinuous { index: 0, .. })
        ));
    }

    #[test]
    fn rejects_oversized_chunks() {
        let rows = chain(&["noop"], &["{}"]);
        let base = entry(&rows[0]);
        let many: Vec<Entry<'_>> = (0..MAX_CHUNK + 1)
            .map(|_| Entry { ..base.clone() })
            .collect();
        assert!(matches!(
            verify_chunk(GENESIS_HASH, &many),
            Err(VerifyError::ChunkTooLarge { .. })
        ));
    }

    #[test]
    fn empty_chunk_returns_expected_prev_hash() {
        assert_eq!(verify_chunk(GENESIS_HASH, &[]).unwrap(), GENESIS_HASH);
    }

    #[test]
    fn detail_serialization_is_compact_and_stable() {
        #[derive(Serialize)]
        struct Payload<'a> {
            pipe_id: &'a str,
            location: &'a str,
        }
        let value = Payload {
            pipe_id: "P-0001",
            location: "A1",
        };
        assert_eq!(
            detail(&value).unwrap(),
            r#"{"pipe_id":"P-0001","location":"A1"}"#
        );
        assert_eq!(detail(&value).unwrap(), detail(&value).unwrap());
    }
}
