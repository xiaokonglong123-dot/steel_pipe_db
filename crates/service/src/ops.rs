//! 运维用例：审计链分批校验、`used_count` 对账、标准规格种子导入。
//!
//! 对应旧设计 §16 的两个 cron 脚本与 §17 的两个缺口（CLI 入口）。

use std::collections::BTreeMap;

use serde::Serialize;
use steelpipe_audit::{Entry, GENESIS_HASH, MAX_CHUNK, verify_chunk};
use steelpipe_infra::{
    DbPool, InfraError,
    repo::{audit as audit_repo, locations, standards},
    time,
    uow::Uow,
};

use crate::actions;
use crate::auth::Identity;

#[derive(Debug, thiserror::Error)]
pub enum OpsError {
    #[error("批次大小 {0} 非法（1..={MAX_CHUNK}）")]
    BadBatch(usize),
    #[error("审计链在第 {at_id} 条断裂：{reason}")]
    ChainBroken { at_id: i64, reason: String },
    #[error("审计链校验尾与链尾表不一致：日志 {log_tip} / 表 {table_tip}")]
    TipMismatch { log_tip: String, table_tip: String },
    #[error("CSV 解析失败：{0}")]
    Parse(String),
    #[error(transparent)]
    Infra(#[from] InfraError),
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifySummary {
    pub entries: i64,
    pub batches: i64,
    pub tip: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DriftFix {
    pub code: String,
    pub before: i64,
    pub after: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SeedSummary {
    pub inserted: usize,
    pub skipped: usize,
}

pub struct OpsService {
    pool: DbPool,
}

impl OpsService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 分批校验全链：跨批用上一批 tip 续接；最后与链尾表比对。
    pub async fn verify_chain(&self, batch: usize) -> Result<VerifySummary, OpsError> {
        if batch == 0 || batch > MAX_CHUNK {
            return Err(OpsError::BadBatch(batch));
        }

        let mut after_id = 0i64;
        let mut expected_prev = GENESIS_HASH.to_owned();
        let mut entries_total = 0i64;
        let mut batches = 0i64;

        loop {
            let rows = audit_repo::page_after(&self.pool, after_id, batch as i64).await?;
            if rows.is_empty() {
                break;
            }
            let entries: Vec<Entry<'_>> = rows.iter().map(|row| row.as_entry()).collect();
            let tip = verify_chunk(&expected_prev, &entries).map_err(|error| {
                let at_id = rows.first().map(|row| row.id).unwrap_or(after_id + 1);
                OpsError::ChainBroken {
                    at_id,
                    reason: error.to_string(),
                }
            })?;
            expected_prev = tip;
            entries_total += rows.len() as i64;
            batches += 1;
            after_id = rows.last().map(|row| row.id).unwrap_or(after_id);
        }

        let table_tip: String =
            sqlx::query_scalar("SELECT last_hash FROM audit_chain_tip WHERE id = 1")
                .fetch_one(&self.pool)
                .await
                .map_err(InfraError::Connect)?;
        if table_tip != expected_prev {
            return Err(OpsError::TipMismatch {
                log_tip: expected_prev,
                table_tip,
            });
        }

        Ok(VerifySummary {
            entries: entries_total,
            batches,
            tip: expected_prev,
        })
    }

    /// 对账：检测 `used_count` 漂移并重算；修复动作写审计。
    pub async fn reconcile(&self, actor: &Identity) -> Result<Vec<DriftFix>, OpsError> {
        let drift = locations::drift(&self.pool).await?;
        let mut fixed = Vec::with_capacity(drift.len());

        for (code, before, _actual) in drift {
            let mut uow = Uow::begin(&self.pool).await?;
            let after = locations::refresh_used_count(uow.conn(), &code).await?;
            let payload = ReconcileDetail {
                code: &code,
                before,
                after,
                by: &actor.username,
            };
            uow.audit(
                Some(actor.user_id),
                actions::RECONCILE,
                &serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_owned()),
            )
            .await?;
            uow.commit().await?;
            fixed.push(DriftFix {
                code,
                before,
                after,
            });
        }
        Ok(fixed)
    }

    /// 从 CSV 种子化 `standard_specs`（已存在的 grade 跳过）。
    pub async fn seed_standards(
        &self,
        actor: &Identity,
        csv: &str,
    ) -> Result<SeedSummary, OpsError> {
        let csv = csv.strip_prefix('\u{feff}').unwrap_or(csv);
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(csv.as_bytes());
        let headers = reader
            .headers()
            .map_err(|error| OpsError::Parse(error.to_string()))?
            .clone();
        let index: BTreeMap<String, usize> = headers
            .iter()
            .enumerate()
            .map(|(position, name)| (name.trim().to_ascii_lowercase(), position))
            .collect();
        if !index.contains_key("grade") {
            return Err(OpsError::Parse("缺少 grade 列".into()));
        }

        let existing: Vec<String> = standards::list(&self.pool)
            .await?
            .into_iter()
            .map(|row| row.grade)
            .collect();
        let known: std::collections::BTreeSet<String> = existing.into_iter().collect();

        let mut inserted = 0usize;
        let mut skipped = 0usize;

        for record in reader.records() {
            let record = record.map_err(|error| OpsError::Parse(error.to_string()))?;
            let grade = record
                .get(*index.get("grade").expect("上面已检查"))
                .unwrap_or("")
                .trim();
            if grade.is_empty() || known.contains(grade) {
                skipped += 1;
                continue;
            }
            let numeric = |name: &str| -> Option<f64> {
                index
                    .get(name)
                    .and_then(|position| record.get(*position))
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .and_then(|raw| raw.parse::<f64>().ok())
            };
            let source = index
                .get("source")
                .and_then(|position| record.get(*position))
                .map(str::trim)
                .filter(|value| !value.is_empty());

            let mut uow = Uow::begin(&self.pool).await?;
            let id = standards::insert(
                uow.conn(),
                grade,
                numeric("outer_diameter_mm"),
                numeric("min_wall_thickness_mm"),
                numeric("max_wall_thickness_mm"),
                numeric("length_tolerance_mm"),
                source,
            )
            .await?;
            let payload = SeedStandardDetail {
                id,
                grade,
                by: &actor.username,
            };
            uow.audit(
                Some(actor.user_id),
                actions::CREATE_STANDARD,
                &serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_owned()),
            )
            .await?;
            uow.commit().await?;
            inserted += 1;
        }

        let _ = time::now_rfc3339();
        Ok(SeedSummary { inserted, skipped })
    }
}

#[derive(Debug, Serialize)]
struct ReconcileDetail<'a> {
    code: &'a str,
    before: i64,
    after: i64,
    by: &'a str,
}

#[derive(Debug, Serialize)]
struct SeedStandardDetail<'a> {
    id: i64,
    grade: &'a str,
    by: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use steelpipe_auth::hash_password;
    use steelpipe_domain::Role;
    use steelpipe_infra::repo::{audit as audit_repo, users};

    struct Fixture {
        pool: DbPool,
        path: std::path::PathBuf,
        service: OpsService,
        actor: Identity,
    }

    async fn fixture(tag: &str) -> Fixture {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("steelpipe-ops-{tag}-{nanos}.db"));
        let pool = steelpipe_infra::connect(&format!("sqlite://{}", path.display()), 8)
            .await
            .unwrap();
        steelpipe_infra::migrate(&pool).await.unwrap();
        let mut uow = Uow::begin(&pool).await.unwrap();
        let hash = hash_password("pw-123456").unwrap();
        let id = users::insert(
            uow.conn(),
            "root",
            &hash,
            "Root",
            Role::Admin,
            &time::now_rfc3339(),
        )
        .await
        .unwrap();
        uow.commit().await.unwrap();
        Fixture {
            service: OpsService::new(pool.clone()),
            actor: Identity {
                user_id: id,
                username: "root".into(),
                display_name: "Root".into(),
                role: Role::Admin,
                token_version: 0,
            },
            pool,
            path,
        }
    }

    #[tokio::test]
    async fn verify_chain_passes_and_detects_tampering() {
        let fx = fixture("verify").await;
        for i in 0..5 {
            let mut uow = Uow::begin(&fx.pool).await.unwrap();
            uow.audit(None, "noop", &format!(r#"{{"i":{i}}}"#))
                .await
                .unwrap();
            uow.commit().await.unwrap();
        }

        let summary = fx.service.verify_chain(2).await.expect("校验应通过");
        assert_eq!(summary.entries, 5);
        assert_eq!(summary.batches, 3);

        // 篡改第二条的 detail
        sqlx::query("UPDATE audit_logs SET detail = '{\"i\":99}' WHERE id = 2")
            .execute(&fx.pool)
            .await
            .unwrap();
        let err = fx.service.verify_chain(10).await.unwrap_err();
        assert!(matches!(err, OpsError::ChainBroken { .. }), "实际：{err:?}");

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn verify_rejects_bad_batch_size() {
        let fx = fixture("batch").await;
        assert!(matches!(
            fx.service.verify_chain(0).await.unwrap_err(),
            OpsError::BadBatch(0)
        ));
        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn reconcile_fixes_drift_and_audits() {
        let fx = fixture("reconcile").await;
        let mut uow = Uow::begin(&fx.pool).await.unwrap();
        locations::insert(uow.conn(), "A1", None, None)
            .await
            .unwrap();
        // 制造漂移：计数 5，实际 0
        locations::adjust_used_count(uow.conn(), "A1", 5)
            .await
            .unwrap();
        uow.commit().await.unwrap();
        assert_eq!(locations::drift(&fx.pool).await.unwrap().len(), 1);

        let fixed = fx.service.reconcile(&fx.actor).await.expect("对账失败");
        assert_eq!(fixed.len(), 1);
        assert_eq!((fixed[0].before, fixed[0].after), (5, 0));
        assert!(locations::drift(&fx.pool).await.unwrap().is_empty());

        let rows = audit_repo::page_after(&fx.pool, 0, 10).await.unwrap();
        assert_eq!(rows[0].action, actions::RECONCILE);

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn seed_standards_skips_existing_and_audits() {
        let fx = fixture("seed").await;
        let mut uow = Uow::begin(&fx.pool).await.unwrap();
        standards::insert(uow.conn(), "L80", Some(88.9), None, None, None, None)
            .await
            .unwrap();
        uow.commit().await.unwrap();

        let csv = "grade,outer_diameter_mm,min_wall_thickness_mm,max_wall_thickness_mm,length_tolerance_mm,source\n\
                   L80,88.9,6.45,7.34,500,API 5CT\n\
                   N80,60.3,5.0,6.0,,\n";
        let summary = fx
            .service
            .seed_standards(&fx.actor, csv)
            .await
            .expect("种子失败");
        assert_eq!((summary.inserted, summary.skipped), (1, 1));

        let all = standards::list(&fx.pool).await.unwrap();
        assert_eq!(all.len(), 2);
        assert!(all.iter().any(|row| row.grade == "N80"));

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }
}
