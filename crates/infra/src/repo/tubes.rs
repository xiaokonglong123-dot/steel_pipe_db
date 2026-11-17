//! 管子仓储。写操作一律 `&mut SqliteConnection`（来自 [crate::uow::Uow]）。
//!
//! 关键语义：状态推进用**带条件的 UPDATE**（`WHERE status = ?`），
//! 影响行数为 0 就是 TOCTOU 冲突 —— 等价旧实现 `guarded_update` 的守卫。

use sqlx::{FromRow, SqliteConnection};
use steelpipe_domain::tube::TubeDraft;
use steelpipe_domain::{QualityMark, TubeStatus, TubeType};

use crate::{DbPool, InfraError};

#[derive(Debug, Clone, FromRow)]
pub struct TubeRow {
    pub id: i64,
    pub pipe_id: String,
    #[sqlx(rename = "type")]
    pub pipe_type: String,
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
    pub status: String,
    pub quality_mark: String,
    pub quality_note: Option<String>,
    pub stock_in_time: Option<String>,
    pub stock_out_time: Option<String>,
    pub compliance_pass: Option<bool>,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl TubeRow {
    pub fn status(&self) -> Option<TubeStatus> {
        self.status.parse().ok()
    }

    pub fn quality(&self) -> Option<QualityMark> {
        self.quality_mark.parse().ok()
    }

    pub fn pipe_type(&self) -> Option<TubeType> {
        self.pipe_type.parse().ok()
    }
}

const COLUMNS: &str =
    "id, pipe_id, type, outer_diameter_mm, wall_thickness_mm, length_m, steel_grade,
    thread_type, api_5ct_cert, manufacturer, batch_no, heat_no, heat_treat_method, location,
    status, quality_mark, quality_note, stock_in_time, stock_out_time, compliance_pass,
    deleted_at, created_at, updated_at";

/// 列表筛选条件。
#[derive(Debug, Clone, Default)]
pub struct TubeFilter {
    pub status: Option<TubeStatus>,
    pub location: Option<String>,
    pub steel_grade: Option<String>,
    pub quality_mark: Option<QualityMark>,
    pub include_deleted: bool,
    pub limit: i64,
    pub offset: i64,
}

impl TubeFilter {
    fn where_clause(&self) -> (String, Vec<String>) {
        let mut parts = Vec::new();
        let mut binds = Vec::new();
        if !self.include_deleted {
            parts.push("deleted_at IS NULL");
        }
        if let Some(status) = self.status {
            parts.push("status = ?");
            binds.push(status.as_str().to_owned());
        }
        if let Some(location) = &self.location {
            parts.push("location = ?");
            binds.push(location.clone());
        }
        if let Some(grade) = &self.steel_grade {
            parts.push("steel_grade = ?");
            binds.push(grade.clone());
        }
        if let Some(mark) = self.quality_mark {
            parts.push("quality_mark = ?");
            binds.push(mark.as_str().to_owned());
        }
        let clause = if parts.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", parts.join(" AND "))
        };
        (clause, binds)
    }
}

pub async fn list(pool: &DbPool, filter: &TubeFilter) -> Result<Vec<TubeRow>, InfraError> {
    let (clause, binds) = filter.where_clause();
    let sql = format!("SELECT {COLUMNS} FROM tubes{clause} ORDER BY id LIMIT ? OFFSET ?");
    let mut query = sqlx::query_as::<_, TubeRow>(&sql);
    for bind in &binds {
        query = query.bind(bind);
    }
    query
        .bind(filter.limit.clamp(1, 500))
        .bind(filter.offset.max(0))
        .fetch_all(pool)
        .await
        .map_err(InfraError::Connect)
}

pub async fn count(pool: &DbPool, filter: &TubeFilter) -> Result<i64, InfraError> {
    let (clause, binds) = filter.where_clause();
    let sql = format!("SELECT count(*) FROM tubes{clause}");
    let mut query = sqlx::query_scalar::<_, i64>(&sql);
    for bind in &binds {
        query = query.bind(bind);
    }
    query.fetch_one(pool).await.map_err(InfraError::Connect)
}

pub async fn find(pool: &DbPool, pipe_id: &str) -> Result<Option<TubeRow>, InfraError> {
    let sql = format!("SELECT {COLUMNS} FROM tubes WHERE pipe_id = ?");
    sqlx::query_as::<_, TubeRow>(&sql)
        .bind(pipe_id)
        .fetch_optional(pool)
        .await
        .map_err(InfraError::Connect)
}

/// 入库：写入管子主表。编号重复 → [InfraError::Conflict]。
#[allow(
    clippy::too_many_arguments,
    reason = "入库字段就是这么多，拆结构体反而更难对照表结构"
)]
pub async fn insert(
    conn: &mut SqliteConnection,
    pipe_id: &str,
    draft: &TubeDraft,
    compliance_pass: bool,
    stock_in_time: &str,
    now: &str,
) -> Result<(), InfraError> {
    sqlx::query(
        "INSERT INTO tubes (pipe_id, type, outer_diameter_mm, wall_thickness_mm, length_m,
            steel_grade, thread_type, api_5ct_cert, manufacturer, batch_no, heat_no,
            heat_treat_method, location, status, quality_mark, quality_note,
            stock_in_time, stock_out_time, compliance_pass, deleted_at, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'in_stock', ?, ?, ?, NULL, ?, NULL, ?, ?)",
    )
    .bind(pipe_id)
    .bind(draft.pipe_type.as_str())
    .bind(draft.outer_diameter_mm)
    .bind(draft.wall_thickness_mm)
    .bind(draft.length_m)
    .bind(draft.steel_grade.trim())
    .bind(&draft.thread_type)
    .bind(&draft.api_5ct_cert)
    .bind(draft.manufacturer.trim())
    .bind(&draft.batch_no)
    .bind(&draft.heat_no)
    .bind(&draft.heat_treat_method)
    .bind(draft.location.trim())
    .bind(draft.quality_mark.as_str())
    .bind(&draft.quality_note)
    .bind(stock_in_time)
    .bind(i64::from(compliance_pass))
    .bind(now)
    .bind(now)
    .execute(conn)
    .await
    .map(|_| ())
    .map_err(|e| InfraError::from_unique(e, "管子编号"))
}

/// 类型扩展表（1:1）。`seamless_pipe_specs` / `screen_pipe_specs`。
pub async fn insert_seamless_spec(
    conn: &mut SqliteConnection,
    pipe_id: &str,
    surface_treatment: Option<&str>,
    end_protector: Option<&str>,
) -> Result<(), InfraError> {
    sqlx::query(
        "INSERT INTO seamless_pipe_specs (pipe_id, surface_treatment, end_protector) VALUES (?, ?, ?)",
    )
    .bind(pipe_id)
    .bind(surface_treatment)
    .bind(end_protector)
    .execute(conn)
    .await
    .map(|_| ())
    .map_err(InfraError::Connect)
}

pub async fn insert_screen_spec(
    conn: &mut SqliteConnection,
    pipe_id: &str,
    specs: &steelpipe_compliance::ScreenPipeParams,
    base_pipe_grade: Option<&str>,
) -> Result<(), InfraError> {
    sqlx::query(
        "INSERT INTO screen_pipe_specs (pipe_id, screen_type, slot_width_mm, slot_density,
            filtration_rating_um, base_pipe_grade, slot_pattern) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(pipe_id)
    .bind(None::<String>)
    .bind(specs.slot_width_mm)
    .bind(specs.slot_density)
    .bind(specs.filtration_rating_um)
    .bind(base_pipe_grade)
    .bind(&specs.slot_pattern)
    .execute(conn)
    .await
    .map(|_| ())
    .map_err(InfraError::Connect)
}

/// 带条件的状态推进：影响行数 0 → [InfraError::Conflict]（TOCTOU）。
pub async fn update_status_guarded(
    conn: &mut SqliteConnection,
    pipe_id: &str,
    from: TubeStatus,
    to: TubeStatus,
    now: &str,
) -> Result<u64, InfraError> {
    let rows =
        sqlx::query("UPDATE tubes SET status = ?, updated_at = ? WHERE pipe_id = ? AND status = ?")
            .bind(to.as_str())
            .bind(now)
            .bind(pipe_id)
            .bind(from.as_str())
            .execute(conn)
            .await
            .map_err(InfraError::Connect)?
            .rows_affected();
    if rows == 0 {
        return Err(InfraError::Conflict("管子状态已被并发修改"));
    }
    Ok(rows)
}

/// 出库：状态推进 + 写 `stock_out_time`（条件同 [update_status_guarded]）。
pub async fn mark_out_stock(
    conn: &mut SqliteConnection,
    pipe_id: &str,
    now: &str,
) -> Result<(), InfraError> {
    let rows = sqlx::query(
        "UPDATE tubes SET status = 'out_stock', stock_out_time = ?, updated_at = ?
         WHERE pipe_id = ? AND status IN ('in_stock', 'in_transfer')",
    )
    .bind(now)
    .bind(now)
    .bind(pipe_id)
    .execute(conn)
    .await
    .map_err(InfraError::Connect)?
    .rows_affected();
    if rows == 0 {
        return Err(InfraError::Conflict("管子状态已被并发修改"));
    }
    Ok(())
}

pub async fn set_location(
    conn: &mut SqliteConnection,
    pipe_id: &str,
    location: &str,
    now: &str,
) -> Result<(), InfraError> {
    sqlx::query("UPDATE tubes SET location = ?, updated_at = ? WHERE pipe_id = ?")
        .bind(location)
        .bind(now)
        .bind(pipe_id)
        .execute(conn)
        .await
        .map_err(InfraError::Connect)?;
    Ok(())
}

pub async fn set_quality(
    conn: &mut SqliteConnection,
    pipe_id: &str,
    mark: QualityMark,
    note: Option<&str>,
    now: &str,
) -> Result<(), InfraError> {
    let rows = sqlx::query(
        "UPDATE tubes SET quality_mark = ?, quality_note = ?, updated_at = ?
         WHERE pipe_id = ? AND deleted_at IS NULL",
    )
    .bind(mark.as_str())
    .bind(note)
    .bind(now)
    .bind(pipe_id)
    .execute(conn)
    .await
    .map_err(InfraError::Connect)?
    .rows_affected();
    if rows == 0 {
        return Err(InfraError::NotFound("管子"));
    }
    Ok(())
}

/// 软删：不物理删除（设计 §17 有意为之）。
pub async fn soft_delete(
    conn: &mut SqliteConnection,
    pipe_id: &str,
    now: &str,
) -> Result<(), InfraError> {
    let rows = sqlx::query(
        "UPDATE tubes SET deleted_at = ?, updated_at = ? WHERE pipe_id = ? AND deleted_at IS NULL",
    )
    .bind(now)
    .bind(now)
    .bind(pipe_id)
    .execute(conn)
    .await
    .map_err(InfraError::Connect)?
    .rows_affected();
    if rows == 0 {
        return Err(InfraError::NotFound("管子"));
    }
    Ok(())
}

/// 交接单历史（详情页用）。
#[derive(Debug, Clone, FromRow)]
pub struct TubeHistoryRow {
    pub transaction_id: i64,
    pub kind: String,
    pub source_location: Option<String>,
    pub target_location: Option<String>,
    pub created_at: String,
    pub operator_id: i64,
    pub remark: Option<String>,
}

pub async fn history(pool: &DbPool, pipe_id: &str) -> Result<Vec<TubeHistoryRow>, InfraError> {
    sqlx::query_as(
        "SELECT t.id AS transaction_id, t.type AS kind, i.source_location, i.target_location,
                t.created_at, t.operator_id, t.remark
         FROM transaction_items i
         JOIN transactions t ON t.id = i.transaction_id
         WHERE i.pipe_id = ? ORDER BY t.id DESC LIMIT 200",
    )
    .bind(pipe_id)
    .fetch_all(pool)
    .await
    .map_err(InfraError::Connect)
}
