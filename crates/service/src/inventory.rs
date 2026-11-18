//! 库存用例：入库（含 API 5CT 合规门禁）、出库、批量出库、调拨、质量标注、软删、详情。
//!
//! 设计不变量（§12）：
//! - 业务写 + 审计行在同一 `BEGIN IMMEDIATE` 事务（[`Uow`] 保证）
//! - 状态推进用带条件 UPDATE；0 行 → 409（TOCTOU）
//! - `used_count` 只随 `in_stock` 管子增减；批量出库整体原子
//!
//! 规格：`docs/spec.md`（D 域）。强制入库（`force`）需 admin，
//! 且额外写一条 `forced_intake` 审计（设计 §11 的缺口在这里补上）。

use serde::Serialize;
use steelpipe_audit::detail;
use steelpipe_compliance::{ScreenPipeParams, TubeParams, validate_tube};
use steelpipe_domain::tube::{TubeDraft, TubeError};
use steelpipe_domain::{QualityMark, TransactionType, TubeStatus, TubeType};
use steelpipe_infra::{
    DbPool, InfraError,
    repo::{locations, standards, transactions, tubes},
    time,
    uow::Uow,
};

use crate::actions;
use crate::auth::Identity;

#[derive(Debug, thiserror::Error)]
pub enum InventoryError {
    #[error("管子 {0} 不存在")]
    TubeNotFound(String),
    #[error("库位 {0} 不存在")]
    LocationNotFound(String),
    #[error("库位 {0} 已停用")]
    LocationInactive(String),
    #[error("源库位与目标库位相同")]
    SameLocation,
    #[error("没有钢级 {0} 的标准规格，无法校验合规")]
    NoStandardSpec(String),
    #[error("合规校验未通过")]
    ComplianceRejected(Vec<steelpipe_compliance::ComplianceCheck>),
    #[error("强制入库需要管理员权限")]
    ForceRequiresAdmin,
    #[error("批量出库列表为空")]
    EmptyBatch,
    #[error("操作被并发修改打断：{0}")]
    Conflict(String),
    #[error(transparent)]
    Domain(#[from] TubeError),
    #[error(transparent)]
    Infra(#[from] InfraError),
}

impl InventoryError {
    fn from_infra(error: InfraError) -> Self {
        match error {
            InfraError::Conflict(message) => Self::Conflict(message.to_owned()),
            InfraError::NotFound(what) => Self::TubeNotFound(what.to_owned()),
            other => Self::Infra(other),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScreenPipeInput {
    pub screen_type: Option<String>,
    pub slot_width_mm: f64,
    pub slot_density: f64,
    pub filtration_rating_um: f64,
    pub slot_pattern: Option<String>,
    pub base_pipe_grade: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SeamlessPipeInput {
    pub surface_treatment: Option<String>,
    pub end_protector: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StockInInput {
    pub pipe_id: String,
    pub pipe_type: TubeType,
    pub outer_diameter_mm: f64,
    pub wall_thickness_mm: f64,
    pub length_m: f64,
    pub steel_grade: String,
    pub thread_type: Option<String>,
    pub api_5ct_cert: Option<String>,
    pub manufacturer: String,
    pub batch_no: Option<String>,
    pub heat_no: Option<String>,
    pub heat_treat_method: Option<String>,
    pub location: String,
    pub quality_mark: QualityMark,
    pub quality_note: Option<String>,
    pub screen: Option<ScreenPipeInput>,
    pub seamless: Option<SeamlessPipeInput>,
    /// 合规未通过时的管理员强制入库（写 `forced_intake` 审计）。
    pub force: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockInOutcome {
    pub pipe_id: String,
    pub compliance_pass: bool,
    pub forced: bool,
    pub location: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockOutOutcome {
    pub pipe_id: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchStockOutOutcome {
    pub pipe_ids: Vec<String>,
    pub transaction_id: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransferOutcome {
    pub pipe_id: String,
    pub from: Option<String>,
    pub to: String,
}

/// 可排序字段（service 视野内的枚举，不暴露仓储类型）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TubeSort {
    PipeId,
    SteelGrade,
    Location,
    Status,
    Quality,
    UpdatedAt,
}

impl TubeSort {
    /// 未知值返回 `None`（按默认 id 排序），不报错。
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "pipe_id" => Some(Self::PipeId),
            "steel_grade" => Some(Self::SteelGrade),
            "location" => Some(Self::Location),
            "status" => Some(Self::Status),
            "quality" => Some(Self::Quality),
            "updated_at" => Some(Self::UpdatedAt),
            _ => None,
        }
    }

    fn to_repo(self) -> tubes::TubeSort {
        match self {
            Self::PipeId => tubes::TubeSort::PipeId,
            Self::SteelGrade => tubes::TubeSort::SteelGrade,
            Self::Location => tubes::TubeSort::Location,
            Self::Status => tubes::TubeSort::Status,
            Self::Quality => tubes::TubeSort::Quality,
            Self::UpdatedAt => tubes::TubeSort::UpdatedAt,
        }
    }
}

/// 列表查询参数（service 层，不暴露仓储类型）。
#[derive(Debug, Clone)]
pub struct TubeQuery {
    pub status: Option<TubeStatus>,
    pub location: Option<String>,
    pub steel_grade: Option<String>,
    pub quality_mark: Option<QualityMark>,
    pub include_deleted: bool,
    pub sort: Option<TubeSort>,
    pub descending: bool,
    pub page: i64,
    pub page_size: i64,
}

impl Default for TubeQuery {
    fn default() -> Self {
        Self {
            status: None,
            location: None,
            steel_grade: None,
            quality_mark: None,
            include_deleted: false,
            sort: None,
            descending: false,
            page: 1,
            page_size: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TubeView {
    pub pipe_id: String,
    pub pipe_type: TubeType,
    pub outer_diameter_mm: f64,
    pub wall_thickness_mm: f64,
    pub length_m: f64,
    pub steel_grade: String,
    pub thread_type: Option<String>,
    pub api_5ct_cert: Option<String>,
    pub manufacturer: String,
    pub batch_no: Option<String>,
    pub heat_no: Option<String>,
    pub heat_treat_method: Option<String>,
    pub location: Option<String>,
    pub status: TubeStatus,
    pub quality_mark: QualityMark,
    pub quality_note: Option<String>,
    pub stock_in_time: Option<String>,
    pub stock_out_time: Option<String>,
    pub compliance_pass: Option<bool>,
    pub deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<tubes::TubeRow> for TubeView {
    fn from(row: tubes::TubeRow) -> Self {
        // 先解析再搬字段：结构体字面量按顺序求值，先 move 后 borrow 会编译失败
        let pipe_type = row.pipe_type().unwrap_or(TubeType::SeamlessPipe);
        let status = row.status().unwrap_or(TubeStatus::OutStock);
        let quality_mark = row.quality().unwrap_or(QualityMark::Normal);
        let deleted = row.deleted_at.is_some();
        Self {
            pipe_id: row.pipe_id,
            pipe_type,
            outer_diameter_mm: row.outer_diameter_mm,
            wall_thickness_mm: row.wall_thickness_mm,
            length_m: row.length_m,
            steel_grade: row.steel_grade,
            thread_type: row.thread_type,
            api_5ct_cert: row.api_5ct_cert,
            manufacturer: row.manufacturer,
            batch_no: row.batch_no,
            heat_no: row.heat_no,
            heat_treat_method: row.heat_treat_method,
            location: row.location,
            status,
            quality_mark,
            quality_note: row.quality_note,
            stock_in_time: row.stock_in_time,
            stock_out_time: row.stock_out_time,
            compliance_pass: row.compliance_pass,
            deleted,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SeamlessSpecView {
    pub surface_treatment: Option<String>,
    pub end_protector: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScreenSpecView {
    pub screen_type: Option<String>,
    pub slot_width_mm: Option<f64>,
    pub slot_density: Option<f64>,
    pub filtration_rating_um: Option<f64>,
    pub base_pipe_grade: Option<String>,
    pub slot_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistoryView {
    pub transaction_id: i64,
    pub kind: String,
    pub source_location: Option<String>,
    pub target_location: Option<String>,
    pub created_at: String,
    pub operator_id: i64,
    pub remark: Option<String>,
}

impl From<tubes::TubeHistoryRow> for HistoryView {
    fn from(row: tubes::TubeHistoryRow) -> Self {
        Self {
            transaction_id: row.transaction_id,
            kind: row.kind,
            source_location: row.source_location,
            target_location: row.target_location,
            created_at: row.created_at,
            operator_id: row.operator_id,
            remark: row.remark,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TubeDetailView {
    pub tube: TubeView,
    pub seamless: Option<SeamlessSpecView>,
    pub screen: Option<ScreenSpecView>,
    pub history: Vec<HistoryView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TubePage {
    pub tubes: Vec<TubeView>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 在给定事务里写入管子主表 + 类型扩展表。
///
/// 入库与盘点盘盈共用（盘盈也必须走同一套写路径，否则扩展表会漏）。
pub(crate) async fn write_tube_rows(
    uow: &mut Uow,
    pipe_id: &str,
    input: &StockInInput,
    compliance_pass: bool,
    now: &str,
) -> Result<(), InventoryError> {
    let draft = TubeDraft {
        pipe_id: pipe_id.to_owned(),
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
    tubes::insert(uow.conn(), pipe_id, &draft, compliance_pass, now, now)
        .await
        .map_err(InventoryError::from_infra)?;

    match input.pipe_type {
        TubeType::SeamlessPipe => {
            let (surface, end) = input
                .seamless
                .as_ref()
                .map(|value| {
                    (
                        value.surface_treatment.as_deref(),
                        value.end_protector.as_deref(),
                    )
                })
                .unwrap_or((None, None));
            tubes::insert_seamless_spec(uow.conn(), pipe_id, surface, end).await?;
        }
        TubeType::ScreenPipe => {
            let screen = input.screen.as_ref();
            let specs = ScreenPipeParams::new(
                screen.map_or(0.0, |value| value.slot_width_mm),
                screen.map_or(0.0, |value| value.slot_density),
                screen.map_or(0.0, |value| value.filtration_rating_um),
                screen.and_then(|value| value.slot_pattern.clone()),
            );
            let base_grade = screen.and_then(|value| value.base_pipe_grade.as_deref());
            tubes::insert_screen_spec(
                uow.conn(),
                pipe_id,
                &specs,
                screen.and_then(|value| value.screen_type.as_deref()),
                base_grade,
            )
            .await?;
        }
    }
    Ok(())
}

pub struct InventoryService {
    pool: DbPool,
}

impl InventoryService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    /// 列表（读侧）：筛选 + 分页，默认不含软删。
    pub async fn list(&self, query: TubeQuery) -> Result<TubePage, InventoryError> {
        let page = query.page.max(1);
        let page_size = query.page_size.clamp(1, 500);
        let filter = tubes::TubeFilter {
            status: query.status,
            location: query.location,
            steel_grade: query.steel_grade,
            quality_mark: query.quality_mark,
            include_deleted: query.include_deleted,
            sort: query.sort.map(TubeSort::to_repo),
            descending: query.descending,
            limit: page_size,
            offset: (page - 1) * page_size,
        };
        let total = tubes::count(&self.pool, &filter).await?;
        let rows = tubes::list(&self.pool, &filter).await?;
        Ok(TubePage {
            tubes: rows.into_iter().map(TubeView::from).collect(),
            total,
            page,
            page_size,
        })
    }

    /// 详情：主表 + 类型扩展 + 流转历史。
    pub async fn detail(&self, pipe_id: &str) -> Result<TubeDetailView, InventoryError> {
        let tube = tubes::find(&self.pool, pipe_id)
            .await?
            .ok_or_else(|| InventoryError::TubeNotFound(pipe_id.to_owned()))?;
        let seamless =
            tubes::seamless_spec(&self.pool, pipe_id)
                .await?
                .map(|row| SeamlessSpecView {
                    surface_treatment: row.surface_treatment,
                    end_protector: row.end_protector,
                });
        let screen = tubes::screen_spec(&self.pool, pipe_id)
            .await?
            .map(|row| ScreenSpecView {
                screen_type: row.screen_type,
                slot_width_mm: row.slot_width_mm,
                slot_density: row.slot_density,
                filtration_rating_um: row.filtration_rating_um,
                base_pipe_grade: row.base_pipe_grade,
                slot_pattern: row.slot_pattern,
            });
        let history = tubes::history(&self.pool, pipe_id)
            .await?
            .into_iter()
            .map(HistoryView::from)
            .collect();
        Ok(TubeDetailView {
            tube: TubeView::from(tube),
            seamless,
            screen,
            history,
        })
    }

    pub async fn history(&self, pipe_id: &str) -> Result<Vec<HistoryView>, InventoryError> {
        Ok(tubes::history(&self.pool, pipe_id)
            .await?
            .into_iter()
            .map(HistoryView::from)
            .collect())
    }

    /// 入库：几何校验 → 标准规格 → 合规五项 → 单事务写管子/扩展表/单据/审计/used_count。
    pub async fn stock_in(
        &self,
        actor: &Identity,
        input: StockInInput,
    ) -> Result<StockInOutcome, InventoryError> {
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
        let pipe_id = draft.validate()?;

        let mut uow = Uow::begin(&self.pool).await?;
        let location = locations::find(uow.conn(), input.location.trim())
            .await?
            .ok_or_else(|| InventoryError::LocationNotFound(input.location.clone()))?;
        if !location.is_active {
            return Err(InventoryError::LocationInactive(location.code));
        }

        let spec = standards::find_by_grade(uow.conn(), input.steel_grade.trim())
            .await?
            .ok_or_else(|| InventoryError::NoStandardSpec(input.steel_grade.clone()))?;

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
        if !result.passed && !input.force {
            return Err(InventoryError::ComplianceRejected(
                result.failures().cloned().collect(),
            ));
        }
        if !result.passed && !actor.role.is_admin() {
            return Err(InventoryError::ForceRequiresAdmin);
        }

        let now = time::now_rfc3339();

        write_tube_rows(&mut uow, &pipe_id, &input, result.passed, &now).await?;

        let transaction_id = transactions::insert(
            uow.conn(),
            TransactionType::In,
            actor.user_id,
            None,
            None,
            &now,
        )
        .await?;
        transactions::insert_item(
            uow.conn(),
            transaction_id,
            &pipe_id,
            None,
            Some(location.code.as_str()),
        )
        .await?;

        locations::adjust_used_count(uow.conn(), &location.code, 1)
            .await
            .map_err(InventoryError::from_infra)?;

        let payload = StockInDetail {
            pipe_id: &pipe_id,
            location: &location.code,
            steel_grade: input.steel_grade.trim(),
            compliance_pass: result.passed,
            forced: !result.passed,
        };
        uow.audit(
            Some(actor.user_id),
            actions::STOCK_IN,
            &detail(&payload).map_err(|error| InventoryError::Conflict(error.to_string()))?,
        )
        .await?;
        if !result.passed {
            let forced_payload = ForcedIntakeDetail {
                pipe_id: &pipe_id,
                steel_grade: input.steel_grade.trim(),
                checks: result.failures().collect(),
            };
            uow.audit(
                Some(actor.user_id),
                actions::FORCED_INTAKE,
                &detail(&forced_payload)
                    .map_err(|error| InventoryError::Conflict(error.to_string()))?,
            )
            .await?;
        }
        uow.commit().await?;

        Ok(StockInOutcome {
            pipe_id,
            compliance_pass: result.passed,
            forced: !result.passed,
            location: location.code,
        })
    }

    /// 单根出库：状态/质量守卫 → 条件 UPDATE → used_count-1 → 单据 → 审计。
    pub async fn stock_out(
        &self,
        actor: &Identity,
        pipe_id: &str,
        remark: Option<&str>,
    ) -> Result<StockOutOutcome, InventoryError> {
        let now = time::now_rfc3339();
        let mut uow = Uow::begin(&self.pool).await?;
        let tube = tubes::find(uow.conn(), pipe_id)
            .await?
            .ok_or_else(|| InventoryError::TubeNotFound(pipe_id.to_owned()))?;
        if tube.deleted_at.is_some() {
            return Err(InventoryError::TubeNotFound(pipe_id.to_owned()));
        }
        let status = tube.status().unwrap_or(TubeStatus::OutStock);
        let quality = tube.quality().unwrap_or(QualityMark::Normal);
        steelpipe_domain::tube::ensure_stock_out_ready(status, quality, actor.role.is_admin())?;

        tubes::mark_out_stock(uow.conn(), pipe_id, &now)
            .await
            .map_err(InventoryError::from_infra)?;
        if let Some(location) = tube.location.as_deref() {
            locations::adjust_used_count(uow.conn(), location, -1)
                .await
                .map_err(InventoryError::from_infra)?;
        }

        let transaction_id = transactions::insert(
            uow.conn(),
            TransactionType::Out,
            actor.user_id,
            None,
            remark,
            &now,
        )
        .await?;
        transactions::insert_item(
            uow.conn(),
            transaction_id,
            pipe_id,
            tube.location.as_deref(),
            None,
        )
        .await?;

        let payload = StockOutDetail {
            pipe_id,
            location: tube.location.as_deref(),
            quality: quality.as_str(),
            transaction_id,
        };
        uow.audit(
            Some(actor.user_id),
            actions::STOCK_OUT,
            &detail(&payload).map_err(|error| InventoryError::Conflict(error.to_string()))?,
        )
        .await?;
        uow.commit().await?;

        Ok(StockOutOutcome {
            pipe_id: pipe_id.to_owned(),
            location: tube.location,
        })
    }

    /// 批量出库：单事务、单单据、每根一条审计；任一根不满足守卫 → 整体回滚。
    pub async fn batch_stock_out(
        &self,
        actor: &Identity,
        pipe_ids: &[String],
        remark: Option<&str>,
    ) -> Result<BatchStockOutOutcome, InventoryError> {
        let unique: Vec<String> = {
            let mut seen = std::collections::BTreeSet::new();
            pipe_ids
                .iter()
                .map(|id| id.trim().to_owned())
                .filter(|id| !id.is_empty())
                .filter(|id| seen.insert(id.clone()))
                .collect()
        };
        if unique.is_empty() {
            return Err(InventoryError::EmptyBatch);
        }

        // 状态校验与实际写入共用同一事务，避免质量标记被并发替换。
        let mut uow = Uow::begin(&self.pool).await?;
        let mut rows = Vec::with_capacity(unique.len());
        for pipe_id in &unique {
            let tube = tubes::find(uow.conn(), pipe_id)
                .await?
                .ok_or_else(|| InventoryError::TubeNotFound(pipe_id.clone()))?;
            if tube.deleted_at.is_some() {
                return Err(InventoryError::TubeNotFound(pipe_id.clone()));
            }
            let status = tube.status().unwrap_or(TubeStatus::OutStock);
            let quality = tube.quality().unwrap_or(QualityMark::Normal);
            steelpipe_domain::tube::ensure_stock_out_ready(status, quality, actor.role.is_admin())?;
            rows.push(tube);
        }

        let now = time::now_rfc3339();

        let transaction_id = transactions::insert(
            uow.conn(),
            TransactionType::Out,
            actor.user_id,
            None,
            remark.or(Some("batch")),
            &now,
        )
        .await?;

        for tube in &rows {
            tubes::mark_out_stock(uow.conn(), &tube.pipe_id, &now)
                .await
                .map_err(InventoryError::from_infra)?;
            if let Some(location) = tube.location.as_deref() {
                locations::adjust_used_count(uow.conn(), location, -1)
                    .await
                    .map_err(InventoryError::from_infra)?;
            }
            transactions::insert_item(
                uow.conn(),
                transaction_id,
                &tube.pipe_id,
                tube.location.as_deref(),
                None,
            )
            .await?;
            let payload = StockOutDetail {
                pipe_id: &tube.pipe_id,
                location: tube.location.as_deref(),
                quality: tube.quality().unwrap_or(QualityMark::Normal).as_str(),
                transaction_id,
            };
            uow.audit(
                Some(actor.user_id),
                actions::BATCH_STOCK_OUT,
                &detail(&payload).map_err(|error| InventoryError::Conflict(error.to_string()))?,
            )
            .await?;
        }
        uow.commit().await?;

        Ok(BatchStockOutOutcome {
            pipe_ids: unique,
            transaction_id,
        })
    }

    /// 调拨：单根、同库位拒绝、目标库位必须存在且启用；used_count 两侧各调一次。
    pub async fn transfer(
        &self,
        actor: &Identity,
        pipe_id: &str,
        target_location: &str,
        remark: Option<&str>,
    ) -> Result<TransferOutcome, InventoryError> {
        let target = target_location.trim();
        let now = time::now_rfc3339();
        let mut uow = Uow::begin(&self.pool).await?;
        let tube = tubes::find(uow.conn(), pipe_id)
            .await?
            .ok_or_else(|| InventoryError::TubeNotFound(pipe_id.to_owned()))?;
        if tube.deleted_at.is_some() {
            return Err(InventoryError::TubeNotFound(pipe_id.to_owned()));
        }
        let status = tube.status().unwrap_or(TubeStatus::OutStock);
        let quality = tube.quality().unwrap_or(QualityMark::Normal);
        steelpipe_domain::tube::ensure_transfer_ready(status, quality, actor.role.is_admin())?;

        if tube.location.as_deref() == Some(target) {
            return Err(InventoryError::SameLocation);
        }
        let target_row = locations::find(uow.conn(), target)
            .await?
            .ok_or_else(|| InventoryError::LocationNotFound(target.to_owned()))?;
        if !target_row.is_active {
            return Err(InventoryError::LocationInactive(target_row.code));
        }

        tubes::set_location(uow.conn(), pipe_id, tube.location.as_deref(), target, &now)
            .await
            .map_err(InventoryError::from_infra)?;
        if let Some(source) = tube.location.as_deref() {
            locations::adjust_used_count(uow.conn(), source, -1)
                .await
                .map_err(InventoryError::from_infra)?;
        }
        locations::adjust_used_count_checked(uow.conn(), target, 1, true)
            .await
            .map_err(InventoryError::from_infra)?;

        let transaction_id = transactions::insert(
            uow.conn(),
            TransactionType::Move,
            actor.user_id,
            None,
            remark,
            &now,
        )
        .await?;
        transactions::insert_item(
            uow.conn(),
            transaction_id,
            pipe_id,
            tube.location.as_deref(),
            Some(target),
        )
        .await?;

        let payload = TransferDetail {
            pipe_id,
            from: tube.location.as_deref(),
            to: target,
        };
        uow.audit(
            Some(actor.user_id),
            actions::TRANSFER,
            &detail(&payload).map_err(|error| InventoryError::Conflict(error.to_string()))?,
        )
        .await?;
        uow.commit().await?;

        Ok(TransferOutcome {
            pipe_id: pipe_id.to_owned(),
            from: tube.location,
            to: target.to_owned(),
        })
    }

    /// 质量标注（D12）：软记录，不动状态；审计 `quality_mark`。
    pub async fn set_quality(
        &self,
        actor: &Identity,
        pipe_id: &str,
        mark: QualityMark,
        note: Option<&str>,
    ) -> Result<(), InventoryError> {
        let now = time::now_rfc3339();
        let mut uow = Uow::begin(&self.pool).await?;
        tubes::set_quality(uow.conn(), pipe_id, mark, note, &now)
            .await
            .map_err(InventoryError::from_infra)?;
        let payload = QualityDetail {
            pipe_id,
            mark: mark.as_str(),
            note,
        };
        uow.audit(
            Some(actor.user_id),
            actions::QUALITY_MARK,
            &detail(&payload).map_err(|error| InventoryError::Conflict(error.to_string()))?,
        )
        .await?;
        uow.commit().await?;
        Ok(())
    }

    /// 软删（不物理删除）。
    pub async fn soft_delete(&self, actor: &Identity, pipe_id: &str) -> Result<(), InventoryError> {
        let now = time::now_rfc3339();
        let mut uow = Uow::begin(&self.pool).await?;
        let tube = tubes::find(uow.conn(), pipe_id)
            .await?
            .ok_or_else(|| InventoryError::TubeNotFound(pipe_id.to_owned()))?;
        tubes::soft_delete(uow.conn(), pipe_id, &now)
            .await
            .map_err(InventoryError::from_infra)?;
        if tube.status() == Some(TubeStatus::InStock) {
            if let Some(location) = tube.location.as_deref() {
                locations::adjust_used_count(uow.conn(), location, -1)
                    .await
                    .map_err(InventoryError::from_infra)?;
            }
        }
        let payload = DeleteDetail { pipe_id };
        uow.audit(
            Some(actor.user_id),
            actions::DELETE_TUBE,
            &detail(&payload).map_err(|error| InventoryError::Conflict(error.to_string()))?,
        )
        .await?;
        uow.commit().await?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct StockInDetail<'a> {
    pipe_id: &'a str,
    location: &'a str,
    steel_grade: &'a str,
    compliance_pass: bool,
    forced: bool,
}

#[derive(Debug, Serialize)]
struct ForcedIntakeDetail<'a> {
    pipe_id: &'a str,
    steel_grade: &'a str,
    checks: Vec<&'a steelpipe_compliance::ComplianceCheck>,
}

#[derive(Debug, Serialize)]
struct StockOutDetail<'a> {
    pipe_id: &'a str,
    location: Option<&'a str>,
    quality: &'a str,
    transaction_id: i64,
}

#[derive(Debug, Serialize)]
struct TransferDetail<'a> {
    pipe_id: &'a str,
    from: Option<&'a str>,
    to: &'a str,
}

#[derive(Debug, Serialize)]
struct QualityDetail<'a> {
    pipe_id: &'a str,
    mark: &'a str,
    note: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct DeleteDetail<'a> {
    pipe_id: &'a str,
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
        service: InventoryService,
        admin: Identity,
        operator: Identity,
    }

    async fn fixture(tag: &str) -> Fixture {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("steelpipe-inv-{tag}-{nanos}.db"));
        let pool = steelpipe_infra::connect(&format!("sqlite://{}", path.display()), 8)
            .await
            .unwrap();
        steelpipe_infra::migrate(&pool).await.unwrap();

        let mut uow = Uow::begin(&pool).await.unwrap();
        locations::insert(uow.conn(), "A1", Some("A1"), Some(100))
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
        let admin_id = users::insert(
            uow.conn(),
            "root",
            &hash,
            "Root",
            Role::Admin,
            &time::now_rfc3339(),
        )
        .await
        .unwrap();
        let operator_id = users::insert(
            uow.conn(),
            "op",
            &hash,
            "Op",
            Role::Operator,
            &time::now_rfc3339(),
        )
        .await
        .unwrap();
        uow.commit().await.unwrap();

        let identity = |user_id: i64, role: Role, name: &str| Identity {
            user_id,
            username: name.to_owned(),
            display_name: name.to_owned(),
            role,
            token_version: 0,
        };

        Fixture {
            service: InventoryService::new(pool.clone()),
            admin: identity(admin_id, Role::Admin, "root"),
            operator: identity(operator_id, Role::Operator, "op"),
            pool,
            path,
        }
    }

    fn input(pipe_id: &str) -> StockInInput {
        StockInInput {
            pipe_id: pipe_id.into(),
            pipe_type: TubeType::SeamlessPipe,
            outer_diameter_mm: 88.9,
            wall_thickness_mm: 7.0,
            length_m: 9.5,
            steel_grade: "L80".into(),
            thread_type: Some("EUE".into()),
            api_5ct_cert: None,
            manufacturer: "宝钢".into(),
            batch_no: None,
            heat_no: None,
            heat_treat_method: None,
            location: "A1".into(),
            quality_mark: QualityMark::Normal,
            quality_note: None,
            screen: None,
            seamless: Some(SeamlessPipeInput {
                surface_treatment: Some("磷化".into()),
                end_protector: None,
            }),
            force: false,
        }
    }

    async fn used_count(fx: &Fixture, code: &str) -> i64 {
        locations::find(&fx.pool, code)
            .await
            .unwrap()
            .unwrap()
            .used_count
    }

    async fn audit_count(fx: &Fixture) -> i64 {
        audit_repo::count(&fx.pool).await.unwrap()
    }

    #[tokio::test]
    async fn stock_in_writes_tube_spec_txn_audit_and_used_count() {
        let fx = fixture("in").await;
        let outcome = fx
            .service
            .stock_in(&fx.operator, input("P-1"))
            .await
            .expect("入库失败");
        assert!(outcome.compliance_pass);
        assert_eq!(used_count(&fx, "A1").await, 1);
        assert_eq!(audit_count(&fx).await, 1);

        let tube = tubes::find(&fx.pool, "P-1").await.unwrap().unwrap();
        assert_eq!(tube.status().unwrap(), TubeStatus::InStock);
        assert_eq!(tube.compliance_pass, Some(true));
        assert!(tube.stock_in_time.is_some());

        let specs: i64 = sqlx::query_scalar("SELECT count(*) FROM seamless_pipe_specs")
            .fetch_one(&fx.pool)
            .await
            .unwrap();
        assert_eq!(specs, 1);

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn stock_in_rejects_non_compliant_tube_without_force() {
        let fx = fixture("reject").await;
        let mut bad = input("P-2");
        bad.wall_thickness_mm = 3.0;
        let err = fx.service.stock_in(&fx.operator, bad).await.unwrap_err();
        match err {
            InventoryError::ComplianceRejected(checks) => {
                assert_eq!(checks.len(), 1);
                assert_eq!(checks[0].check_name, "wall_thickness_range");
            }
            other => panic!("期望合规拒绝，实际：{other:?}"),
        }
        assert_eq!(used_count(&fx, "A1").await, 0, "拒绝后不得占用库位");
        assert_eq!(audit_count(&fx).await, 0);
        assert!(tubes::find(&fx.pool, "P-2").await.unwrap().is_none());

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn force_intake_requires_admin_and_writes_extra_audit() {
        let fx = fixture("force").await;
        let mut bad = input("P-3");
        bad.wall_thickness_mm = 3.0;
        bad.force = true;

        let err = fx
            .service
            .stock_in(&fx.operator, bad.clone())
            .await
            .unwrap_err();
        assert!(matches!(err, InventoryError::ForceRequiresAdmin));

        let outcome = fx
            .service
            .stock_in(&fx.admin, bad)
            .await
            .expect("管理员强制入库应成功");
        assert!(!outcome.compliance_pass && outcome.forced);
        let actions: Vec<String> = sqlx::query_scalar("SELECT action FROM audit_logs ORDER BY id")
            .fetch_all(&fx.pool)
            .await
            .unwrap();
        assert_eq!(actions, [actions::STOCK_IN, actions::FORCED_INTAKE]);

        let tube = tubes::find(&fx.pool, "P-3").await.unwrap().unwrap();
        assert_eq!(tube.compliance_pass, Some(false));

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn stock_in_requires_known_grade_and_active_location() {
        let fx = fixture("grade").await;
        let mut unknown_grade = input("P-4");
        unknown_grade.steel_grade = "X99".into();
        assert!(matches!(
            fx.service
                .stock_in(&fx.operator, unknown_grade)
                .await
                .unwrap_err(),
            InventoryError::NoStandardSpec(_)
        ));

        let mut bad_location = input("P-5");
        bad_location.location = "Z9".into();
        assert!(matches!(
            fx.service
                .stock_in(&fx.operator, bad_location)
                .await
                .unwrap_err(),
            InventoryError::LocationNotFound(_)
        ));

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn stock_out_guards_status_and_releases_used_count() {
        let fx = fixture("out").await;
        fx.service
            .stock_in(&fx.operator, input("P-6"))
            .await
            .unwrap();
        assert_eq!(used_count(&fx, "A1").await, 1);

        fx.service
            .stock_out(&fx.operator, "P-6", Some("发货"))
            .await
            .expect("出库失败");
        assert_eq!(used_count(&fx, "A1").await, 0);
        let tube = tubes::find(&fx.pool, "P-6").await.unwrap().unwrap();
        assert_eq!(tube.status().unwrap(), TubeStatus::OutStock);
        assert!(tube.stock_out_time.is_some());

        // 二次出库 → 409 语义
        let err = fx
            .service
            .stock_out(&fx.operator, "P-6", None)
            .await
            .unwrap_err();
        assert!(
            matches!(err, InventoryError::Domain(TubeError::NotInStock(_))),
            "实际：{err:?}"
        );

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn problematic_tube_needs_admin_to_leave_stock() {
        let fx = fixture("problem").await;
        let mut mark = input("P-7");
        mark.quality_mark = QualityMark::Problematic;
        fx.service.stock_in(&fx.operator, mark).await.unwrap();

        let err = fx
            .service
            .stock_out(&fx.operator, "P-7", None)
            .await
            .unwrap_err();
        assert!(
            matches!(
                err,
                InventoryError::Domain(TubeError::ProblematicRequiresAdmin)
            ),
            "实际：{err:?}"
        );
        fx.service
            .stock_out(&fx.admin, "P-7", None)
            .await
            .expect("管理员应能出库");

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn batch_stock_out_is_atomic() {
        let fx = fixture("batch").await;
        fx.service
            .stock_in(&fx.operator, input("P-8"))
            .await
            .unwrap();
        fx.service
            .stock_in(&fx.operator, input("P-9"))
            .await
            .unwrap();
        // 先出掉一根，让批量里有一根不满足守卫
        fx.service
            .stock_out(&fx.operator, "P-9", None)
            .await
            .unwrap();
        let audits_before = audit_count(&fx).await;

        let err = fx
            .service
            .batch_stock_out(&fx.operator, &["P-8".into(), "P-9".into()], None)
            .await
            .unwrap_err();
        assert!(matches!(err, InventoryError::Domain(_)), "实际：{err:?}");

        // P-8 必须仍在库，且没有新审计/新单据
        let tube = tubes::find(&fx.pool, "P-8").await.unwrap().unwrap();
        assert_eq!(tube.status().unwrap(), TubeStatus::InStock);
        assert_eq!(audit_count(&fx).await, audits_before);
        let txns: i64 = sqlx::query_scalar("SELECT count(*) FROM transactions")
            .fetch_one(&fx.pool)
            .await
            .unwrap();
        assert_eq!(
            txns, 3,
            "两次入库 + 一次出库 = 3 张单据；失败的批量不得新增"
        );

        // 成功批量：一条单据、每根一条审计
        fx.service
            .batch_stock_out(&fx.operator, &["P-8".into()], None)
            .await
            .expect("批量出库应成功");
        assert_eq!(used_count(&fx, "A1").await, 0);
        assert_eq!(audit_count(&fx).await, audits_before + 1);

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn transfer_moves_counts_and_rejects_same_location() {
        let fx = fixture("transfer").await;
        fx.service
            .stock_in(&fx.operator, input("P-10"))
            .await
            .unwrap();

        let err = fx
            .service
            .transfer(&fx.operator, "P-10", "A1", None)
            .await
            .unwrap_err();
        assert!(matches!(err, InventoryError::SameLocation));

        fx.service
            .transfer(&fx.operator, "P-10", "A2", Some("倒库"))
            .await
            .expect("调拨失败");
        assert_eq!(used_count(&fx, "A1").await, 0);
        assert_eq!(used_count(&fx, "A2").await, 1);
        let tube = tubes::find(&fx.pool, "P-10").await.unwrap().unwrap();
        assert_eq!(tube.location.as_deref(), Some("A2"));
        assert_eq!(tube.status().unwrap(), TubeStatus::InStock);

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn quality_mark_and_soft_delete_write_audit() {
        let fx = fixture("quality").await;
        fx.service
            .stock_in(&fx.operator, input("P-11"))
            .await
            .unwrap();
        fx.service
            .set_quality(
                &fx.operator,
                "P-11",
                QualityMark::Problematic,
                Some("壁厚超差"),
            )
            .await
            .unwrap();
        let tube = tubes::find(&fx.pool, "P-11").await.unwrap().unwrap();
        assert_eq!(tube.quality().unwrap(), QualityMark::Problematic);

        fx.service.soft_delete(&fx.admin, "P-11").await.unwrap();
        assert_eq!(
            used_count(&fx, "A1").await,
            0,
            "软删在库管子要释放 used_count"
        );
        let row = tubes::find(&fx.pool, "P-11").await.unwrap().unwrap();
        assert!(row.deleted_at.is_some());

        let actions: Vec<String> = sqlx::query_scalar("SELECT action FROM audit_logs ORDER BY id")
            .fetch_all(&fx.pool)
            .await
            .unwrap();
        assert_eq!(
            actions,
            [
                actions::STOCK_IN,
                actions::QUALITY_MARK,
                actions::DELETE_TUBE
            ]
        );

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }
}
