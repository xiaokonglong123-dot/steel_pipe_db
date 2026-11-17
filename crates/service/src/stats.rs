//! 仪表盘聚合：KPI + 最近审计。只读，不写审计（避免每次刷新污染链）。

use serde::Serialize;
use steelpipe_infra::{
    DbPool, InfraError,
    repo::{audit as audit_repo, stats},
};

#[derive(Debug, thiserror::Error)]
pub enum StatsError {
    #[error(transparent)]
    Infra(#[from] InfraError),
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct TubeCounts {
    pub total: i64,
    pub in_stock: i64,
    pub in_transfer: i64,
    pub out_stock: i64,
    pub missing: i64,
    pub problematic: i64,
    pub scrapped: i64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct LocationStats {
    pub total: i64,
    pub active: i64,
    pub used: i64,
    pub capacity: i64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct TodayStats {
    pub inbound: i64,
    pub outbound: i64,
    pub transfers: i64,
    pub checks: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditBrief {
    pub id: i64,
    pub user_id: Option<i64>,
    pub action: String,
    pub ts: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditStats {
    pub entries: i64,
    pub seq: i64,
    pub tip: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardView {
    pub tubes: TubeCounts,
    pub locations: LocationStats,
    pub today: TodayStats,
    pub recent_audit: Vec<AuditBrief>,
    pub audit: AuditStats,
}

pub struct StatsService {
    pool: DbPool,
}

impl StatsService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 一次取全量快照，供仪表盘单次渲染。
    pub async fn snapshot(&self) -> Result<DashboardView, StatsError> {
        let mut tubes = TubeCounts::default();
        for (status, count) in stats::tube_counts_by_status(&self.pool).await? {
            tubes.total += count;
            match status.as_str() {
                "in_stock" => tubes.in_stock = count,
                "in_transfer" => tubes.in_transfer = count,
                "out_stock" => tubes.out_stock = count,
                "missing" => tubes.missing = count,
                _ => {}
            }
        }
        for (mark, count) in stats::tube_counts_by_quality(&self.pool).await? {
            match mark.as_str() {
                "problematic" => tubes.problematic = count,
                "scrapped" => tubes.scrapped = count,
                _ => {}
            }
        }

        let location_summary = stats::location_summary(&self.pool).await?;
        let locations = LocationStats {
            total: location_summary.total,
            active: location_summary.active,
            used: location_summary.used,
            capacity: location_summary.capacity,
        };

        let mut today = TodayStats::default();
        for (kind, count) in stats::transaction_counts_since(&self.pool, &today_start()).await? {
            match kind.as_str() {
                "in" => today.inbound = count,
                "out" => today.outbound = count,
                "move" => today.transfers = count,
                "check" => today.checks = count,
                _ => {}
            }
        }

        let recent_audit = audit_repo::last_n(&self.pool, 6)
            .await?
            .into_iter()
            .map(|row| AuditBrief {
                id: row.id,
                user_id: row.user_id,
                action: row.action,
                ts: row.ts,
            })
            .collect();

        let entries = audit_repo::count(&self.pool).await?;
        let (seq, tip) = audit_repo::chain_tip(&self.pool).await?;

        Ok(DashboardView {
            tubes,
            locations,
            today,
            recent_audit,
            audit: AuditStats { entries, seq, tip },
        })
    }
}

/// 本地时区当天 00:00 的 UTC 表示，格式与 `created_at` 逐字符可比。
fn today_start() -> String {
    use chrono::{Local, TimeZone, Utc};
    let now = Local::now();
    let start = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|naive| Local.from_local_datetime(&naive).single())
        .unwrap_or(now);
    start
        .with_timezone(&Utc)
        .to_rfc3339_opts(chrono::SecondsFormat::Micros, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use steelpipe_auth::hash_password;
    use steelpipe_domain::Role;
    use steelpipe_domain::TubeType;
    use steelpipe_domain::tube::TubeDraft;
    use steelpipe_domain::{QualityMark, TransactionType};
    use steelpipe_infra::{
        repo::{locations, transactions, tubes as tube_repo, users},
        time,
        uow::Uow,
    };

    struct Fixture {
        pool: DbPool,
        path: std::path::PathBuf,
        service: StatsService,
    }

    async fn fixture(tag: &str) -> Fixture {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("steelpipe-stats-{tag}-{nanos}.db"));
        let pool = steelpipe_infra::connect(&format!("sqlite://{}", path.display()), 8)
            .await
            .unwrap();
        steelpipe_infra::migrate(&pool).await.unwrap();
        Fixture {
            service: StatsService::new(pool.clone()),
            pool,
            path,
        }
    }

    fn draft(pipe_id: &str) -> TubeDraft {
        TubeDraft {
            pipe_id: pipe_id.into(),
            pipe_type: TubeType::SeamlessPipe,
            outer_diameter_mm: 88.9,
            wall_thickness_mm: 7.0,
            length_m: 9.5,
            steel_grade: "L80".into(),
            thread_type: None,
            manufacturer: "宝钢".into(),
            batch_no: None,
            heat_no: None,
            heat_treat_method: None,
            location: "A1".into(),
            api_5ct_cert: None,
            quality_mark: QualityMark::Normal,
            quality_note: None,
        }
    }

    #[tokio::test]
    async fn snapshot_aggregates_counts_and_recent_audit() {
        let fx = fixture("agg").await;
        let now = time::now_rfc3339();

        let mut uow = Uow::begin(&fx.pool).await.unwrap();
        locations::insert(uow.conn(), "A1", None, Some(100))
            .await
            .unwrap();
        locations::insert(uow.conn(), "A2", None, None)
            .await
            .unwrap();
        locations::adjust_used_count(uow.conn(), "A1", 2)
            .await
            .unwrap();
        let hash = hash_password("pw-123456").unwrap();
        let user_id = users::insert(uow.conn(), "root", &hash, "Root", Role::Admin, &now)
            .await
            .unwrap();
        for (pipe_id, mark) in [
            ("P-1", QualityMark::Normal),
            ("P-2", QualityMark::Problematic),
        ] {
            let mut d = draft(pipe_id);
            d.quality_mark = mark;
            tube_repo::insert(uow.conn(), pipe_id, &d, true, &now, &now)
                .await
                .unwrap();
        }
        tube_repo::update_status_guarded(
            uow.conn(),
            "P-2",
            steelpipe_domain::TubeStatus::InStock,
            steelpipe_domain::TubeStatus::Missing,
            &now,
        )
        .await
        .unwrap();
        transactions::insert(uow.conn(), TransactionType::In, user_id, None, None, &now)
            .await
            .unwrap();
        transactions::insert(uow.conn(), TransactionType::Out, user_id, None, None, &now)
            .await
            .unwrap();
        uow.audit(Some(user_id), "stock_in", "{}").await.unwrap();
        uow.audit(Some(user_id), "stock_out", "{}").await.unwrap();
        uow.commit().await.unwrap();

        let view = fx.service.snapshot().await.expect("快照失败");
        assert_eq!(view.tubes.total, 2);
        assert_eq!(view.tubes.in_stock, 1);
        assert_eq!(view.tubes.missing, 1);
        assert_eq!(view.tubes.problematic, 1);
        assert_eq!(view.locations.total, 2);
        assert_eq!(view.locations.active, 2);
        assert_eq!(view.locations.used, 2);
        assert_eq!(view.locations.capacity, 100);
        assert_eq!(view.today.inbound, 1);
        assert_eq!(view.today.outbound, 1);
        assert_eq!(view.recent_audit.len(), 2);
        assert_eq!(view.recent_audit.last().unwrap().action, "stock_out");
        assert_eq!(view.audit.entries, 2);
        assert_eq!(view.audit.seq, 2);
        assert_eq!(view.audit.tip.len(), 64);

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn empty_database_returns_zeroed_snapshot() {
        let fx = fixture("empty").await;
        let view = fx.service.snapshot().await.expect("快照失败");
        assert_eq!(view.tubes.total, 0);
        assert_eq!(view.today.inbound, 0);
        assert!(view.recent_audit.is_empty());
        assert_eq!(view.audit.entries, 0);
        assert_eq!(view.audit.tip.len(), 64);

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn yesterday_transactions_are_not_counted_as_today() {
        let fx = fixture("today").await;
        let yesterday = (chrono::Utc::now() - chrono::Duration::days(1))
            .to_rfc3339_opts(chrono::SecondsFormat::Micros, false);
        let mut uow = Uow::begin(&fx.pool).await.unwrap();
        transactions::insert(uow.conn(), TransactionType::In, 1, None, None, &yesterday)
            .await
            .unwrap();
        uow.commit().await.unwrap();

        let view = fx.service.snapshot().await.expect("快照失败");
        assert_eq!(view.today.inbound, 0, "昨天的单据不应计入今日");

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }
}
