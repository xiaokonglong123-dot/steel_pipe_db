//! 标准规格仓储（`standard_specs`）。grade 唯一。

use sqlx::{FromRow, Sqlite, SqliteConnection};

use crate::{DbPool, InfraError};

#[derive(Debug, Clone, FromRow)]
pub struct StandardRow {
    pub id: i64,
    pub grade: String,
    pub outer_diameter_mm: Option<f64>,
    pub min_wall_thickness_mm: Option<f64>,
    pub max_wall_thickness_mm: Option<f64>,
    pub length_tolerance_mm: Option<f64>,
    pub source: Option<String>,
}

const COLUMNS: &str = "id, grade, outer_diameter_mm, min_wall_thickness_mm, max_wall_thickness_mm,
    length_tolerance_mm, source";

impl StandardRow {
    /// 转成合规判定所需的纯结构。
    pub fn to_spec(&self) -> steelpipe_compliance::StandardSpec {
        steelpipe_compliance::StandardSpec {
            grade: self.grade.clone(),
            outer_diameter_mm: self.outer_diameter_mm,
            min_wall_thickness_mm: self.min_wall_thickness_mm,
            max_wall_thickness_mm: self.max_wall_thickness_mm,
            length_tolerance_mm: self.length_tolerance_mm,
        }
    }
}

pub async fn list(pool: &DbPool) -> Result<Vec<StandardRow>, InfraError> {
    let sql = format!("SELECT {COLUMNS} FROM standard_specs ORDER BY grade");
    sqlx::query_as::<_, StandardRow>(&sql)
        .fetch_all(pool)
        .await
        .map_err(InfraError::Connect)
}

pub async fn find_by_grade<'e, E>(
    executor: E,
    grade: &str,
) -> Result<Option<StandardRow>, InfraError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let sql = format!("SELECT {COLUMNS} FROM standard_specs WHERE grade = ?");
    sqlx::query_as::<_, StandardRow>(&sql)
        .bind(grade)
        .fetch_optional(executor)
        .await
        .map_err(InfraError::Connect)
}

pub async fn find_by_id<'e, E>(executor: E, id: i64) -> Result<Option<StandardRow>, InfraError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let sql = format!("SELECT {COLUMNS} FROM standard_specs WHERE id = ?");
    sqlx::query_as::<_, StandardRow>(&sql)
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(InfraError::Connect)
}

#[allow(clippy::too_many_arguments, reason = "与表结构一一对应")]
pub async fn insert(
    conn: &mut SqliteConnection,
    grade: &str,
    outer_diameter_mm: Option<f64>,
    min_wall_thickness_mm: Option<f64>,
    max_wall_thickness_mm: Option<f64>,
    length_tolerance_mm: Option<f64>,
    source: Option<&str>,
) -> Result<i64, InfraError> {
    sqlx::query_scalar(
        "INSERT INTO standard_specs (grade, outer_diameter_mm, min_wall_thickness_mm,
            max_wall_thickness_mm, length_tolerance_mm, source)
         VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(grade)
    .bind(outer_diameter_mm)
    .bind(min_wall_thickness_mm)
    .bind(max_wall_thickness_mm)
    .bind(length_tolerance_mm)
    .bind(source)
    .fetch_one(conn)
    .await
    .map_err(|e| InfraError::from_unique(e, "钢级规格"))
}

#[allow(clippy::too_many_arguments, reason = "与表结构一一对应")]
pub async fn update(
    conn: &mut SqliteConnection,
    id: i64,
    grade: &str,
    outer_diameter_mm: Option<f64>,
    min_wall_thickness_mm: Option<f64>,
    max_wall_thickness_mm: Option<f64>,
    length_tolerance_mm: Option<f64>,
    source: Option<&str>,
) -> Result<(), InfraError> {
    let rows = sqlx::query(
        "UPDATE standard_specs SET grade = ?, outer_diameter_mm = ?, min_wall_thickness_mm = ?,
            max_wall_thickness_mm = ?, length_tolerance_mm = ?, source = ? WHERE id = ?",
    )
    .bind(grade)
    .bind(outer_diameter_mm)
    .bind(min_wall_thickness_mm)
    .bind(max_wall_thickness_mm)
    .bind(length_tolerance_mm)
    .bind(source)
    .bind(id)
    .execute(conn)
    .await
    .map_err(|e| InfraError::from_unique(e, "钢级规格"))?
    .rows_affected();
    if rows == 0 {
        return Err(InfraError::NotFound("钢级规格"));
    }
    Ok(())
}

/// 硬删（设计 §17 有意为之：审计留删前快照）。
pub async fn delete(conn: &mut SqliteConnection, id: i64) -> Result<(), InfraError> {
    let rows = sqlx::query("DELETE FROM standard_specs WHERE id = ?")
        .bind(id)
        .execute(conn)
        .await
        .map_err(InfraError::Connect)?
        .rows_affected();
    if rows == 0 {
        return Err(InfraError::NotFound("钢级规格"));
    }
    Ok(())
}
