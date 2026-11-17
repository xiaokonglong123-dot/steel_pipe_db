//! 盘点用例：建单、逐根结论、盘盈、取消、确认（原子应用差异）、找回。
//!
//! 设计 §5.3：
//! - 盘亏用独立状态 `missing`（不复用 `out_stock`）
//! - 盘盈录入实际信息后必须走 API 5CT 合规门（与入库同一套写路径）
//! - 位置异常触发移库
//! - `confirm` 在**单个事务**里应用全部差异；任一项失败整体回滚、session 保持 `open`
//! - 只有 confirm / recover 写审计

use serde::Serialize;
use steelpipe_audit::detail;
use steelpipe_compliance::{ScreenPipeParams, TubeParams, validate_tube};
use steelpipe_domain::tube::TubeDraft;
use steelpipe_domain::{CheckResult, CheckStatus, QualityMark, TransactionType, TubeStatus};
use steelpipe_infra::{
    DbPool, InfraError,
    repo::{checks, locations, standards, transactions, tubes},
    time,
    uow::Uow,
};

use crate::actions;
use crate::auth::Identity;
use crate::inventory::{StockInInput, write_tube_rows};

#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    #[error("盘点单 {0} 不存在")]
    NotFound(i64),
    #[error("盘点单不是 open 状态")]
    NotOpen,
    #[error("库位 {0} 不存在或已停用")]
    LocationUnavailable(String),
    #[error("管子 {0} 不在该库位的在库清单中")]
    TubeNotInScope(String),
    #[error("`moved` 结论必须给出与盘点库位不同的实际位置")]
    InvalidMoved,
    #[error("没有钢级 {0} 的标准规格，无法校验盘盈合规")]
    NoStandardSpec(String),
    #[error("盘盈合规校验未通过")]
    ComplianceRejected(Vec<steelpipe_compliance::ComplianceCheck>),
    #[error("操作被并发修改打断：{0}")]
    Conflict(String),
    #[error(transparent)]
    Domain(#[from] steelpipe_domain::tube::TubeError),
    #[error(transparent)]
    Infra(#[from] InfraError),
}

impl CheckError {
    fn from_infra(error: InfraError) -> Self {
        match error {
            InfraError::Conflict(message) => Self::Conflict(message.to_owned()),
            InfraError::NotFound(_) => Self::NotFound(0),
            other => Self::Infra(other),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckSessionView {
    pub id: i64,
    pub location: String,
    pub status: CheckStatus,
    pub created_by: i64,
    pub confirmed_by: Option<i64>,
    pub created_at: String,
    pub confirmed_at: Option<String>,
    pub remark: Option<String>,
}

impl From<checks::CheckSessionRow> for CheckSessionView {
    fn from(row: checks::CheckSessionRow) -> Self {
        Self {
            id: row.id,
            location: row.location,
            status: row
                .status
                .parse::<CheckStatus>()
                .unwrap_or(CheckStatus::Open),
            created_by: row.created_by,
            confirmed_by: row.confirmed_by,
            created_at: row.created_at,
            confirmed_at: row.confirmed_at,
            remark: row.remark,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckItemView {
    pub id: i64,
    pub pipe_id: String,
    pub result: CheckResult,
    pub actual_location: Option<String>,
    pub surplus: Option<StockInInput>,
    pub created_at: String,
}

impl From<checks::CheckItemRow> for CheckItemView {
    fn from(row: checks::CheckItemRow) -> Self {
        Self {
            id: row.id,
            pipe_id: row.pipe_id,
            result: row.result.parse::<CheckResult>().unwrap_or(CheckResult::Ok),
            actual_location: row.actual_location,
            surplus: row
                .surplus_data
                .as_deref()
                .and_then(|raw| serde_json::from_str::<StockInInput>(raw).ok()),
            created_at: row.created_at,
        }
    }
}

/// 盘点页需要的「系统认为在该库位的在库管」清单。
#[derive(Debug, Clone, Serialize)]
pub struct ExpectedTubeView {
    pub pipe_id: String,
    pub steel_grade: String,
    pub outer_diameter_mm: f64,
    pub wall_thickness_mm: f64,
    pub length_m: f64,
    pub quality_mark: QualityMark,
}

pub struct CheckDetailView {
    pub session: CheckSessionView,
    pub items: Vec<CheckItemView>,
    pub expected: Vec<ExpectedTubeView>,
    pub active_locations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfirmOutcome {
    pub check_id: i64,
    pub missing: i64,
    pub surplus: i64,
    pub moved: i64,
    pub transaction_id: i64,
}

pub struct CheckService {
    pool: DbPool,
}

impl CheckService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<CheckSessionView>, CheckError> {
        Ok(checks::list_sessions(&self.pool, 200, 0)
            .await?
            .into_iter()
            .map(CheckSessionView::from)
            .collect())
    }

    pub async fn create(
        &self,
        actor: &Identity,
        location: &str,
        remark: Option<&str>,
    ) -> Result<CheckSessionView, CheckError> {
        let location_code = location.trim();
        let row = locations::find(&self.pool, location_code)
            .await?
            .ok_or_else(|| CheckError::LocationUnavailable(location_code.to_owned()))?;
        if !row.is_active {
            return Err(CheckError::LocationUnavailable(row.code));
        }

        let now = time::now_rfc3339();
        let mut uow = Uow::begin(&self.pool).await?;
        let id = checks::insert_session(uow.conn(), &row.code, actor.user_id, remark, &now).await?;
        uow.commit().await?;

        Ok(CheckSessionView {
            id,
            location: row.code,
            status: CheckStatus::Open,
            created_by: actor.user_id,
            confirmed_by: None,
            created_at: now,
            confirmed_at: None,
            remark: remark.map(ToOwned::to_owned),
        })
    }

    pub async fn detail(&self, check_id: i64) -> Result<CheckDetailView, CheckError> {
        let session = checks::find_session(&self.pool, check_id)
            .await?
            .ok_or(CheckError::NotFound(check_id))?;
        let items = checks::items(&self.pool, check_id).await?;
        let expected_rows = tubes::list(
            &self.pool,
            &tubes::TubeFilter {
                status: Some(TubeStatus::InStock),
                location: Some(session.location.clone()),
                limit: 500,
                ..Default::default()
            },
        )
        .await?;
        let active_locations = locations::list(&self.pool)
            .await?
            .into_iter()
            .filter(|row| row.is_active)
            .map(|row| row.code)
            .collect();

        Ok(CheckDetailView {
            session: CheckSessionView::from(session),
            items: items.into_iter().map(CheckItemView::from).collect(),
            expected: expected_rows
                .into_iter()
                .map(|row| {
                    // 先解析再搬字段（结构体字面量按序求值）
                    let quality_mark = row.quality().unwrap_or(QualityMark::Normal);
                    ExpectedTubeView {
                        pipe_id: row.pipe_id,
                        steel_grade: row.steel_grade,
                        outer_diameter_mm: row.outer_diameter_mm,
                        wall_thickness_mm: row.wall_thickness_mm,
                        length_m: row.length_m,
                        quality_mark,
                    }
                })
                .collect(),
            active_locations,
        })
    }

    /// 逐根结论：`ok` / `missing` / `moved`。管子必须仍在该库位且 `in_stock`。
    pub async fn mark_item(
        &self,
        actor: &Identity,
        check_id: i64,
        pipe_id: &str,
        result: CheckResult,
        actual_location: Option<&str>,
    ) -> Result<(), CheckError> {
        let session = self.require_open(check_id).await?;
        let tube = tubes::find(&self.pool, pipe_id)
            .await?
            .ok_or_else(|| CheckError::TubeNotInScope(pipe_id.to_owned()))?;
        if tube.deleted_at.is_some()
            || tube.status() != Some(TubeStatus::InStock)
            || tube.location.as_deref() != Some(session.location.as_str())
        {
            return Err(CheckError::TubeNotInScope(pipe_id.to_owned()));
        }

        let actual = actual_location
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if result == CheckResult::Moved {
            let Some(target) = actual else {
                return Err(CheckError::InvalidMoved);
            };
            if target == session.location {
                return Err(CheckError::InvalidMoved);
            }
            let target_row = locations::find(&self.pool, target)
                .await?
                .ok_or_else(|| CheckError::LocationUnavailable(target.to_owned()))?;
            if !target_row.is_active {
                return Err(CheckError::LocationUnavailable(target_row.code));
            }
        }

        let now = time::now_rfc3339();
        let mut uow = Uow::begin(&self.pool).await?;
        checks::upsert_item(
            uow.conn(),
            check_id,
            pipe_id,
            result.as_str(),
            if result == CheckResult::Moved {
                actual
            } else {
                None
            },
            None,
            &now,
        )
        .await?;
        uow.commit().await?;
        let _ = actor;
        Ok(())
    }

    /// 盘盈：先暂存实际信息，确认时才真正入库并过合规门。
    pub async fn add_surplus(
        &self,
        _actor: &Identity,
        check_id: i64,
        input: StockInInput,
    ) -> Result<(), CheckError> {
        let session = self.require_open(check_id).await?;
        let pipe_id = input.pipe_id.trim().to_owned();
        if pipe_id.is_empty() {
            return Err(CheckError::TubeNotInScope(String::new()));
        }
        if tubes::find(&self.pool, &pipe_id).await?.is_some() {
            return Err(CheckError::Conflict("该编号的管子已存在".into()));
        }

        let mut payload = input;
        payload.location = session.location.clone();
        let raw = serde_json::to_string(&payload)
            .map_err(|error| CheckError::Conflict(error.to_string()))?;

        let now = time::now_rfc3339();
        let mut uow = Uow::begin(&self.pool).await?;
        checks::upsert_item(
            uow.conn(),
            check_id,
            &pipe_id,
            CheckResult::Surplus.as_str(),
            None,
            Some(&raw),
            &now,
        )
        .await?;
        uow.commit().await?;
        Ok(())
    }

    pub async fn remove_item(&self, check_id: i64, item_id: i64) -> Result<(), CheckError> {
        self.require_open(check_id).await?;
        let mut uow = Uow::begin(&self.pool).await?;
        checks::delete_item(uow.conn(), check_id, item_id)
            .await
            .map_err(CheckError::from_infra)?;
        uow.commit().await?;
        Ok(())
    }

    pub async fn cancel(&self, actor: &Identity, check_id: i64) -> Result<(), CheckError> {
        let now = time::now_rfc3339();
        let mut uow = Uow::begin(&self.pool).await?;
        checks::close_session(
            uow.conn(),
            check_id,
            CheckStatus::Cancelled.as_str(),
            actor.user_id,
            &now,
        )
        .await
        .map_err(CheckError::from_infra)?;
        uow.commit().await?;
        Ok(())
    }

    /// 确认：单片事务应用全部差异；盘盈走合规门；写一条 `check_confirm` 审计。
    pub async fn confirm(
        &self,
        actor: &Identity,
        check_id: i64,
    ) -> Result<ConfirmOutcome, CheckError> {
        let session = self.require_open(check_id).await?;
        let location = session.location.clone();
        let items = checks::items(&self.pool, check_id).await?;

        // 盘盈的合规校验必须先做完，避免写了一半才失败
        let mut surplus_inputs: Vec<(String, StockInInput)> = Vec::new();
        for item in &items {
            if item.result != CheckResult::Surplus.as_str() {
                continue;
            }
            let raw = item
                .surplus_data
                .as_deref()
                .ok_or_else(|| CheckError::Conflict("盘盈缺少实际信息".into()))?;
            let input: StockInInput = serde_json::from_str(raw)
                .map_err(|error| CheckError::Conflict(format!("盘盈数据损坏：{error}")))?;
            self.validate_surplus(&input).await?;
            surplus_inputs.push((item.pipe_id.clone(), input));
        }

        let now = time::now_rfc3339();
        let mut uow = Uow::begin(&self.pool).await?;

        let mut missing_n = 0i64;
        let mut moved_n = 0i64;
        let mut surplus_n = 0i64;

        for item in &items {
            match item
                .result
                .parse::<CheckResult>()
                .unwrap_or(CheckResult::Ok)
            {
                CheckResult::Ok => continue,
                CheckResult::Missing => {
                    tubes::mark_missing(uow.conn(), &item.pipe_id, &location, &now)
                        .await
                        .map_err(CheckError::from_infra)?;
                    locations::adjust_used_count(uow.conn(), &location, -1)
                        .await
                        .map_err(CheckError::from_infra)?;
                    missing_n += 1;
                }
                CheckResult::Moved => {
                    let target = item
                        .actual_location
                        .as_deref()
                        .ok_or(CheckError::InvalidMoved)?;
                    tubes::move_location_guarded(
                        uow.conn(),
                        &item.pipe_id,
                        &location,
                        target,
                        &now,
                    )
                    .await
                    .map_err(CheckError::from_infra)?;
                    locations::adjust_used_count(uow.conn(), &location, -1)
                        .await
                        .map_err(CheckError::from_infra)?;
                    locations::adjust_used_count(uow.conn(), target, 1)
                        .await
                        .map_err(CheckError::from_infra)?;
                    moved_n += 1;
                }
                CheckResult::Surplus => {
                    let (pipe_id, input) = surplus_inputs
                        .iter()
                        .find(|(pipe_id, _)| pipe_id == &item.pipe_id)
                        .ok_or_else(|| CheckError::Conflict("盘盈数据缺失".into()))?;
                    write_tube_rows(&mut uow, pipe_id, input, true, &now)
                        .await
                        .map_err(|error| CheckError::Conflict(error.to_string()))?;
                    locations::adjust_used_count(uow.conn(), &location, 1)
                        .await
                        .map_err(CheckError::from_infra)?;
                    surplus_n += 1;
                }
            }
        }

        let transaction_id = transactions::insert(
            uow.conn(),
            TransactionType::Check,
            actor.user_id,
            None,
            Some(&format!("check {check_id}")),
            &now,
        )
        .await?;
        for item in &items {
            let result = item
                .result
                .parse::<CheckResult>()
                .unwrap_or(CheckResult::Ok);
            let (source, target) = match result {
                CheckResult::Ok => continue,
                CheckResult::Missing => (Some(location.as_str()), None),
                CheckResult::Moved => (Some(location.as_str()), item.actual_location.as_deref()),
                CheckResult::Surplus => (None, Some(location.as_str())),
            };
            transactions::insert_item(uow.conn(), transaction_id, &item.pipe_id, source, target)
                .await?;
        }

        checks::close_session(
            uow.conn(),
            check_id,
            CheckStatus::Confirmed.as_str(),
            actor.user_id,
            &now,
        )
        .await
        .map_err(CheckError::from_infra)?;

        let payload = ConfirmDetail {
            check_id,
            location: &location,
            missing: missing_n,
            moved: moved_n,
            surplus: surplus_n,
        };
        uow.audit(
            Some(actor.user_id),
            actions::CHECK_CONFIRM,
            &detail(&payload).map_err(|error| CheckError::Conflict(error.to_string()))?,
        )
        .await?;
        uow.commit().await?;

        Ok(ConfirmOutcome {
            check_id,
            missing: missing_n,
            surplus: surplus_n,
            moved: moved_n,
            transaction_id,
        })
    }

    /// 找回：`missing` → `in_stock`，计数 +1，写 `check_recover` 审计。
    pub async fn recover(&self, actor: &Identity, pipe_id: &str) -> Result<(), CheckError> {
        let now = time::now_rfc3339();
        let mut uow = Uow::begin(&self.pool).await?;
        let location = tubes::mark_in_stock_from_missing(uow.conn(), pipe_id, &now)
            .await
            .map_err(CheckError::from_infra)?;
        if let Some(code) = location.as_deref() {
            locations::adjust_used_count(uow.conn(), code, 1)
                .await
                .map_err(CheckError::from_infra)?;
        }
        transactions::insert(
            uow.conn(),
            TransactionType::Check,
            actor.user_id,
            None,
            Some("recover"),
            &now,
        )
        .await?;
        let payload = RecoverDetail { pipe_id };
        uow.audit(
            Some(actor.user_id),
            actions::CHECK_RECOVER,
            &detail(&payload).map_err(|error| CheckError::Conflict(error.to_string()))?,
        )
        .await?;
        uow.commit().await?;
        Ok(())
    }

    async fn require_open(&self, check_id: i64) -> Result<checks::CheckSessionRow, CheckError> {
        let session = checks::find_session(&self.pool, check_id)
            .await?
            .ok_or(CheckError::NotFound(check_id))?;
        if session.status != CheckStatus::Open.as_str() {
            return Err(CheckError::NotOpen);
        }
        Ok(session)
    }

    async fn validate_surplus(&self, input: &StockInInput) -> Result<(), CheckError> {
        let draft = TubeDraft {
            pipe_id: input.pipe_id.clone(),
            pipe_type: input.pipe_type,
            outer_diameter_mm: input.outer_diameter_mm,
            wall_thickness_mm: input.wall_thickness_mm,
            length_m: input.length_m,
            steel_grade: input.steel_grade.clone(),
            thread_type: input.thread_type.clone(),
            manufacturer: input.manufacturer.clone(),
            batch_no: input.batch_no.clone(),
            heat_no: input.heat_no.clone(),
            heat_treat_method: input.heat_treat_method.clone(),
            location: input.location.clone(),
            api_5ct_cert: input.api_5ct_cert.clone(),
            quality_mark: input.quality_mark,
            quality_note: input.quality_note.clone(),
        };
        draft.validate()?;

        let spec = standards::find_by_grade(&self.pool, input.steel_grade.trim())
            .await?
            .ok_or_else(|| CheckError::NoStandardSpec(input.steel_grade.clone()))?;
        let params = TubeParams {
            pipe_type: input.pipe_type,
            outer_diameter_mm: input.outer_diameter_mm,
            wall_thickness_mm: input.wall_thickness_mm,
            length_m: input.length_m,
            steel_grade: input.steel_grade.trim().to_owned(),
            thread_type: input.thread_type.clone(),
            screen_specs: input.screen.as_ref().map(|screen| {
                ScreenPipeParams::new(
                    screen.slot_width_mm,
                    screen.slot_density,
                    screen.filtration_rating_um,
                    screen.slot_pattern.clone(),
                )
            }),
        };
        let result = validate_tube(&params, &spec.to_spec());
        if !result.passed {
            return Err(CheckError::ComplianceRejected(
                result.failures().cloned().collect(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct ConfirmDetail<'a> {
    check_id: i64,
    location: &'a str,
    missing: i64,
    moved: i64,
    surplus: i64,
}

#[derive(Debug, Serialize)]
struct RecoverDetail<'a> {
    pipe_id: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use steelpipe_auth::hash_password;
    use steelpipe_domain::{Role, TubeType};
    use steelpipe_infra::repo::{audit as audit_repo, users};

    struct Fixture {
        pool: DbPool,
        path: std::path::PathBuf,
        service: CheckService,
        actor: Identity,
        inventory: crate::InventoryService,
    }

    async fn fixture(tag: &str) -> Fixture {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("steelpipe-check-{tag}-{nanos}.db"));
        let pool = steelpipe_infra::connect(&format!("sqlite://{}", path.display()), 8)
            .await
            .unwrap();
        steelpipe_infra::migrate(&pool).await.unwrap();

        let mut uow = Uow::begin(&pool).await.unwrap();
        locations::insert(uow.conn(), "A1", Some("A1"), None)
            .await
            .unwrap();
        locations::insert(uow.conn(), "A2", Some("A2"), None)
            .await
            .unwrap();
        standards::insert(
            uow.conn(),
            "L80",
            Some(88.9),
            Some(6.45),
            Some(7.34),
            None,
            None,
        )
        .await
        .unwrap();
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
            service: CheckService::new(pool.clone()),
            inventory: crate::InventoryService::new(pool.clone()),
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

    fn stock_input(pipe_id: &str, wall: f64) -> StockInInput {
        StockInInput {
            pipe_id: pipe_id.into(),
            pipe_type: TubeType::SeamlessPipe,
            outer_diameter_mm: 88.9,
            wall_thickness_mm: wall,
            length_m: 9.5,
            steel_grade: "L80".into(),
            thread_type: None,
            api_5ct_cert: None,
            manufacturer: "宝钢".into(),
            batch_no: None,
            heat_no: None,
            heat_treat_method: None,
            location: "A1".into(),
            quality_mark: QualityMark::Normal,
            quality_note: None,
            screen: None,
            seamless: None,
            force: false,
        }
    }

    async fn tube_status(fx: &Fixture, pipe_id: &str) -> (String, Option<String>) {
        let row = tubes::find(&fx.pool, pipe_id).await.unwrap().unwrap();
        (row.status, row.location)
    }

    async fn used(fx: &Fixture, code: &str) -> i64 {
        locations::find(&fx.pool, code)
            .await
            .unwrap()
            .unwrap()
            .used_count
    }

    #[tokio::test]
    async fn confirm_applies_missing_and_moved_atomically() {
        let fx = fixture("confirm").await;
        fx.inventory
            .stock_in(&fx.actor, stock_input("P-1", 7.0))
            .await
            .unwrap();
        fx.inventory
            .stock_in(&fx.actor, stock_input("P-2", 7.0))
            .await
            .unwrap();
        assert_eq!(used(&fx, "A1").await, 2);

        let session = fx
            .service
            .create(&fx.actor, "A1", Some("月度盘点"))
            .await
            .unwrap();
        fx.service
            .mark_item(&fx.actor, session.id, "P-1", CheckResult::Missing, None)
            .await
            .unwrap();
        fx.service
            .mark_item(&fx.actor, session.id, "P-2", CheckResult::Moved, Some("A2"))
            .await
            .unwrap();

        let outcome = fx
            .service
            .confirm(&fx.actor, session.id)
            .await
            .expect("确认失败");
        assert_eq!((outcome.missing, outcome.moved, outcome.surplus), (1, 1, 0));

        assert_eq!(tube_status(&fx, "P-1").await.0, "missing");
        assert_eq!(tube_status(&fx, "P-2").await.1.as_deref(), Some("A2"));
        assert_eq!(used(&fx, "A1").await, 0);
        assert_eq!(used(&fx, "A2").await, 1);

        let detail = fx.service.detail(session.id).await.unwrap();
        assert_eq!(detail.session.status, CheckStatus::Confirmed);

        // 只有 confirm 写审计（建单/核对不写）
        let rows = audit_repo::page_after(&fx.pool, 0, 50).await.unwrap();
        let actions: Vec<&str> = rows.iter().map(|row| row.action.as_str()).collect();
        assert_eq!(actions.last().copied(), Some(actions::CHECK_CONFIRM));
        assert_eq!(
            actions
                .iter()
                .filter(|action| **action == actions::CHECK_CONFIRM)
                .count(),
            1
        );

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn surplus_goes_through_compliance_gate() {
        let fx = fixture("surplus").await;
        let session = fx.service.create(&fx.actor, "A1", None).await.unwrap();

        // 不合规盘盈：确认时才校验，必须整体失败且 session 保持 open
        fx.service
            .add_surplus(&fx.actor, session.id, stock_input("P-9", 3.0))
            .await
            .unwrap();
        let err = fx.service.confirm(&fx.actor, session.id).await.unwrap_err();
        assert!(
            matches!(err, CheckError::ComplianceRejected(_)),
            "实际：{err:?}"
        );
        let detail = fx.service.detail(session.id).await.unwrap();
        assert_eq!(
            detail.session.status,
            CheckStatus::Open,
            "失败必须保持 open"
        );
        assert!(tubes::find(&fx.pool, "P-9").await.unwrap().is_none());
        assert_eq!(used(&fx, "A1").await, 0);
        assert_eq!(audit_repo::count(&fx.pool).await.unwrap(), 0);

        // 合规盘盈：确认后入库并计数 +1
        fx.service
            .remove_item(session.id, detail.items[0].id)
            .await
            .unwrap();
        fx.service
            .add_surplus(&fx.actor, session.id, stock_input("P-9", 7.0))
            .await
            .unwrap();
        let outcome = fx
            .service
            .confirm(&fx.actor, session.id)
            .await
            .expect("确认失败");
        assert_eq!(outcome.surplus, 1);
        assert_eq!(used(&fx, "A1").await, 1);
        assert_eq!(tube_status(&fx, "P-9").await.0, "in_stock");

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn missing_tube_can_be_recovered() {
        let fx = fixture("recover").await;
        fx.inventory
            .stock_in(&fx.actor, stock_input("P-3", 7.0))
            .await
            .unwrap();
        let session = fx.service.create(&fx.actor, "A1", None).await.unwrap();
        fx.service
            .mark_item(&fx.actor, session.id, "P-3", CheckResult::Missing, None)
            .await
            .unwrap();
        fx.service.confirm(&fx.actor, session.id).await.unwrap();
        assert_eq!(tube_status(&fx, "P-3").await.0, "missing");
        assert_eq!(used(&fx, "A1").await, 0);

        fx.service
            .recover(&fx.actor, "P-3")
            .await
            .expect("找回失败");
        assert_eq!(tube_status(&fx, "P-3").await.0, "in_stock");
        assert_eq!(used(&fx, "A1").await, 1);

        // 非 missing 状态再找回 → 冲突
        let err = fx.service.recover(&fx.actor, "P-3").await.unwrap_err();
        assert!(matches!(err, CheckError::Conflict(_)), "实际：{err:?}");

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn mark_item_scope_and_validation() {
        let fx = fixture("scope").await;
        fx.inventory
            .stock_in(&fx.actor, stock_input("P-4", 7.0))
            .await
            .unwrap();
        // 另一库位也有管，不能出现在 A1 的盘点里
        let mut other = stock_input("P-5", 7.0);
        other.location = "A2".into();
        fx.inventory.stock_in(&fx.actor, other).await.unwrap();

        let session = fx.service.create(&fx.actor, "A1", None).await.unwrap();
        let err = fx
            .service
            .mark_item(&fx.actor, session.id, "P-5", CheckResult::Missing, None)
            .await
            .unwrap_err();
        assert!(
            matches!(err, CheckError::TubeNotInScope(_)),
            "实际：{err:?}"
        );

        // moved 必须给出不同于盘点库位的目标
        let err = fx
            .service
            .mark_item(&fx.actor, session.id, "P-4", CheckResult::Moved, Some("A1"))
            .await
            .unwrap_err();
        assert!(matches!(err, CheckError::InvalidMoved));

        let err = fx
            .service
            .mark_item(&fx.actor, session.id, "P-4", CheckResult::Moved, None)
            .await
            .unwrap_err();
        assert!(matches!(err, CheckError::InvalidMoved));

        // 已确认的盘点单不能再改
        fx.service.confirm(&fx.actor, session.id).await.unwrap();
        let err = fx
            .service
            .mark_item(&fx.actor, session.id, "P-4", CheckResult::Ok, None)
            .await
            .unwrap_err();
        assert!(matches!(err, CheckError::NotOpen), "实际：{err:?}");

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn cancelling_keeps_stock_untouched() {
        let fx = fixture("cancel").await;
        fx.inventory
            .stock_in(&fx.actor, stock_input("P-6", 7.0))
            .await
            .unwrap();
        let session = fx.service.create(&fx.actor, "A1", None).await.unwrap();
        fx.service
            .mark_item(&fx.actor, session.id, "P-6", CheckResult::Missing, None)
            .await
            .unwrap();
        fx.service.cancel(&fx.actor, session.id).await.unwrap();

        let detail = fx.service.detail(session.id).await.unwrap();
        assert_eq!(detail.session.status, CheckStatus::Cancelled);
        assert_eq!(
            tube_status(&fx, "P-6").await.0,
            "in_stock",
            "取消不得改动库存"
        );
        assert_eq!(used(&fx, "A1").await, 1);
        assert_eq!(
            audit_repo::count(&fx.pool).await.unwrap(),
            1,
            "只有入库审计"
        );

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }
}
