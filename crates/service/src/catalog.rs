//! 目录用例：库位（创建/改名/停用/启用）与标准规格（CRUD）。
//!
//! 全部写操作单事务 + 同事务审计；标准规格硬删但审计里留删前快照（设计 §17）。

use serde::Serialize;
use steelpipe_audit::detail;
use steelpipe_infra::{
    DbPool, InfraError,
    repo::{locations, standards},
    uow::Uow,
};

use crate::actions;
use crate::auth::Identity;

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("{0}不存在")]
    NotFound(&'static str),
    #[error("{0}已存在")]
    Conflict(&'static str),
    #[error("{0}")]
    Invalid(String),
    #[error(transparent)]
    Infra(#[from] InfraError),
}

impl CatalogError {
    fn from_infra(error: InfraError) -> Self {
        match error {
            InfraError::NotFound(what) => Self::NotFound(what),
            InfraError::Conflict(what) => Self::Conflict(what),
            other => Self::Infra(other),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LocationView {
    pub code: String,
    pub name: Option<String>,
    pub used_count: i64,
    pub capacity: Option<i64>,
    pub is_active: bool,
}

impl From<locations::LocationRow> for LocationView {
    fn from(row: locations::LocationRow) -> Self {
        Self {
            code: row.code,
            name: row.name,
            used_count: row.used_count,
            capacity: row.capacity,
            is_active: row.is_active,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StandardView {
    pub id: i64,
    pub grade: String,
    pub outer_diameter_mm: Option<f64>,
    pub min_wall_thickness_mm: Option<f64>,
    pub max_wall_thickness_mm: Option<f64>,
    pub length_tolerance_mm: Option<f64>,
    pub source: Option<String>,
}

impl From<standards::StandardRow> for StandardView {
    fn from(row: standards::StandardRow) -> Self {
        Self {
            id: row.id,
            grade: row.grade,
            outer_diameter_mm: row.outer_diameter_mm,
            min_wall_thickness_mm: row.min_wall_thickness_mm,
            max_wall_thickness_mm: row.max_wall_thickness_mm,
            length_tolerance_mm: row.length_tolerance_mm,
            source: row.source,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StandardInput {
    pub grade: String,
    pub outer_diameter_mm: Option<f64>,
    pub min_wall_thickness_mm: Option<f64>,
    pub max_wall_thickness_mm: Option<f64>,
    pub length_tolerance_mm: Option<f64>,
    pub source: Option<String>,
}

impl StandardInput {
    fn validate(&self) -> Result<(), CatalogError> {
        let grade = self.grade.trim();
        if grade.is_empty() {
            return Err(CatalogError::Invalid("钢级不能为空".into()));
        }
        if grade.chars().count() > 32 {
            return Err(CatalogError::Invalid("钢级长度上限 32".into()));
        }
        for (label, value) in [
            ("外径", self.outer_diameter_mm),
            ("最小壁厚", self.min_wall_thickness_mm),
            ("最大壁厚", self.max_wall_thickness_mm),
        ] {
            if let Some(value) = value {
                if !value.is_finite() || value <= 0.0 {
                    return Err(CatalogError::Invalid(format!("{label}必须为正数")));
                }
            }
        }
        if let Some(tolerance) = self.length_tolerance_mm {
            if !tolerance.is_finite() || tolerance < 0.0 {
                return Err(CatalogError::Invalid("长度公差不能为负".into()));
            }
        }
        if let (Some(min), Some(max)) = (self.min_wall_thickness_mm, self.max_wall_thickness_mm) {
            if min > max {
                return Err(CatalogError::Invalid("最小壁厚不能大于最大壁厚".into()));
            }
        }
        Ok(())
    }
}

pub struct CatalogService {
    pool: DbPool,
}

impl CatalogService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    // ---------------- 库位 ----------------

    pub async fn list_locations(&self) -> Result<Vec<LocationView>, CatalogError> {
        Ok(locations::list(&self.pool)
            .await?
            .into_iter()
            .map(LocationView::from)
            .collect())
    }

    pub async fn create_location(
        &self,
        actor: &Identity,
        code: &str,
        name: Option<&str>,
        capacity: Option<i64>,
    ) -> Result<LocationView, CatalogError> {
        let code = code.trim();
        if code.is_empty() {
            return Err(CatalogError::Invalid("库位编号不能为空".into()));
        }
        if code.chars().count() > 32 {
            return Err(CatalogError::Invalid("库位编号长度上限 32".into()));
        }
        if capacity.is_some_and(|value| value < 0) {
            return Err(CatalogError::Invalid("容量不能为负".into()));
        }

        let mut uow = Uow::begin(&self.pool).await?;
        locations::insert(uow.conn(), code, name, capacity)
            .await
            .map_err(CatalogError::from_infra)?;
        let payload = LocationDetail {
            code,
            name,
            capacity,
            by: &actor.username,
        };
        uow.audit(
            Some(actor.user_id),
            actions::CREATE_LOCATION,
            &detail(&payload).map_err(|error| CatalogError::Invalid(error.to_string()))?,
        )
        .await?;
        uow.commit().await?;

        Ok(LocationView {
            code: code.to_owned(),
            name: name.map(ToOwned::to_owned),
            used_count: 0,
            capacity,
            is_active: true,
        })
    }

    pub async fn update_location(
        &self,
        actor: &Identity,
        code: &str,
        name: Option<&str>,
        capacity: Option<i64>,
    ) -> Result<(), CatalogError> {
        if capacity.is_some_and(|value| value < 0) {
            return Err(CatalogError::Invalid("容量不能为负".into()));
        }
        let mut uow = Uow::begin(&self.pool).await?;
        locations::rename(uow.conn(), code, name, capacity)
            .await
            .map_err(CatalogError::from_infra)?;
        let payload = LocationDetail {
            code,
            name,
            capacity,
            by: &actor.username,
        };
        uow.audit(
            Some(actor.user_id),
            actions::UPDATE_LOCATION,
            &detail(&payload).map_err(|error| CatalogError::Invalid(error.to_string()))?,
        )
        .await?;
        uow.commit().await?;
        Ok(())
    }

    /// 停用/启用。停用后不能再入库或作为调拨目标；重新启用在此实现（旧实现缺入口）。
    pub async fn set_location_active(
        &self,
        actor: &Identity,
        code: &str,
        active: bool,
    ) -> Result<(), CatalogError> {
        let mut uow = Uow::begin(&self.pool).await?;
        locations::set_active(uow.conn(), code, active)
            .await
            .map_err(CatalogError::from_infra)?;
        let payload = LocationActiveDetail {
            code,
            active,
            by: &actor.username,
        };
        uow.audit(
            Some(actor.user_id),
            if active {
                actions::UPDATE_LOCATION
            } else {
                actions::DEACTIVATE_LOCATION
            },
            &detail(&payload).map_err(|error| CatalogError::Invalid(error.to_string()))?,
        )
        .await?;
        uow.commit().await?;
        Ok(())
    }

    // ---------------- 标准规格 ----------------

    pub async fn list_standards(&self) -> Result<Vec<StandardView>, CatalogError> {
        Ok(standards::list(&self.pool)
            .await?
            .into_iter()
            .map(StandardView::from)
            .collect())
    }

    pub async fn create_standard(
        &self,
        actor: &Identity,
        input: StandardInput,
    ) -> Result<StandardView, CatalogError> {
        input.validate()?;
        let mut uow = Uow::begin(&self.pool).await?;
        let id = standards::insert(
            uow.conn(),
            input.grade.trim(),
            input.outer_diameter_mm,
            input.min_wall_thickness_mm,
            input.max_wall_thickness_mm,
            input.length_tolerance_mm,
            input.source.as_deref(),
        )
        .await
        .map_err(CatalogError::from_infra)?;
        let payload = StandardDetail {
            id,
            grade: input.grade.trim(),
            by: &actor.username,
        };
        uow.audit(
            Some(actor.user_id),
            actions::CREATE_STANDARD,
            &detail(&payload).map_err(|error| CatalogError::Invalid(error.to_string()))?,
        )
        .await?;
        uow.commit().await?;

        Ok(StandardView {
            id,
            grade: input.grade.trim().to_owned(),
            outer_diameter_mm: input.outer_diameter_mm,
            min_wall_thickness_mm: input.min_wall_thickness_mm,
            max_wall_thickness_mm: input.max_wall_thickness_mm,
            length_tolerance_mm: input.length_tolerance_mm,
            source: input.source,
        })
    }

    pub async fn update_standard(
        &self,
        actor: &Identity,
        id: i64,
        input: StandardInput,
    ) -> Result<(), CatalogError> {
        input.validate()?;
        let mut uow = Uow::begin(&self.pool).await?;
        standards::update(
            uow.conn(),
            id,
            input.grade.trim(),
            input.outer_diameter_mm,
            input.min_wall_thickness_mm,
            input.max_wall_thickness_mm,
            input.length_tolerance_mm,
            input.source.as_deref(),
        )
        .await
        .map_err(CatalogError::from_infra)?;
        let payload = StandardDetail {
            id,
            grade: input.grade.trim(),
            by: &actor.username,
        };
        uow.audit(
            Some(actor.user_id),
            actions::UPDATE_STANDARD,
            &detail(&payload).map_err(|error| CatalogError::Invalid(error.to_string()))?,
        )
        .await?;
        uow.commit().await?;
        Ok(())
    }

    /// 硬删 + 审计里留删前快照（旧实现同样如此）。
    pub async fn delete_standard(&self, actor: &Identity, id: i64) -> Result<(), CatalogError> {
        let snapshot = standards::find_by_id(&self.pool, id)
            .await?
            .ok_or(CatalogError::NotFound("钢级规格"))?;

        let mut uow = Uow::begin(&self.pool).await?;
        standards::delete(uow.conn(), id)
            .await
            .map_err(CatalogError::from_infra)?;
        let payload = StandardDeleteDetail {
            id,
            snapshot: StandardView::from(snapshot.clone()),
            by: &actor.username,
        };
        uow.audit(
            Some(actor.user_id),
            actions::DELETE_STANDARD,
            &detail(&payload).map_err(|error| CatalogError::Invalid(error.to_string()))?,
        )
        .await?;
        uow.commit().await?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct LocationDetail<'a> {
    code: &'a str,
    name: Option<&'a str>,
    capacity: Option<i64>,
    by: &'a str,
}

#[derive(Debug, Serialize)]
struct LocationActiveDetail<'a> {
    code: &'a str,
    active: bool,
    by: &'a str,
}

#[derive(Debug, Serialize)]
struct StandardDetail<'a> {
    id: i64,
    grade: &'a str,
    by: &'a str,
}

#[derive(Debug, Serialize)]
struct StandardDeleteDetail<'a> {
    id: i64,
    snapshot: StandardView,
    by: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use steelpipe_auth::hash_password;
    use steelpipe_domain::Role;
    use steelpipe_infra::repo::{audit as audit_repo, users};
    use steelpipe_infra::time;

    struct Fixture {
        pool: DbPool,
        path: std::path::PathBuf,
        service: CatalogService,
        actor: Identity,
    }

    async fn fixture(tag: &str) -> Fixture {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("steelpipe-cat-{tag}-{nanos}.db"));
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
            service: CatalogService::new(pool.clone()),
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
    async fn location_lifecycle_writes_audit() {
        let fx = fixture("loc").await;
        fx.service
            .create_location(&fx.actor, "A1", Some("一号库"), Some(50))
            .await
            .expect("创建库位失败");
        assert_eq!(fx.service.list_locations().await.unwrap().len(), 1);

        let err = fx
            .service
            .create_location(&fx.actor, "A1", None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, CatalogError::Conflict(_)));

        fx.service
            .update_location(&fx.actor, "A1", Some("一号库房"), Some(80))
            .await
            .unwrap();
        fx.service
            .set_location_active(&fx.actor, "A1", false)
            .await
            .unwrap();
        let views = fx.service.list_locations().await.unwrap();
        assert!(!views[0].is_active);
        assert_eq!(views[0].name.as_deref(), Some("一号库房"));
        assert_eq!(views[0].capacity, Some(80));

        // 重新启用（旧实现的缺口）
        fx.service
            .set_location_active(&fx.actor, "A1", true)
            .await
            .unwrap();
        assert!(fx.service.list_locations().await.unwrap()[0].is_active);

        let actions: Vec<String> = sqlx::query_scalar("SELECT action FROM audit_logs ORDER BY id")
            .fetch_all(&fx.pool)
            .await
            .unwrap();
        assert_eq!(
            actions,
            [
                actions::CREATE_LOCATION,
                actions::UPDATE_LOCATION,
                actions::DEACTIVATE_LOCATION,
                actions::UPDATE_LOCATION
            ]
        );

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn location_validation_rejects_bad_input() {
        let fx = fixture("loc-bad").await;
        assert!(matches!(
            fx.service
                .create_location(&fx.actor, "  ", None, None)
                .await
                .unwrap_err(),
            CatalogError::Invalid(_)
        ));
        assert!(matches!(
            fx.service
                .create_location(&fx.actor, "A1", None, Some(-1))
                .await
                .unwrap_err(),
            CatalogError::Invalid(_)
        ));
        assert_eq!(audit_repo::count(&fx.pool).await.unwrap(), 0);
        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn standard_crud_audits_delete_snapshot() {
        let fx = fixture("std").await;
        let view = fx
            .service
            .create_standard(
                &fx.actor,
                StandardInput {
                    grade: "L80".into(),
                    outer_diameter_mm: Some(88.9),
                    min_wall_thickness_mm: Some(6.45),
                    max_wall_thickness_mm: Some(7.34),
                    length_tolerance_mm: None,
                    source: Some("API 5CT".into()),
                },
            )
            .await
            .expect("创建规格失败");

        fx.service
            .update_standard(
                &fx.actor,
                view.id,
                StandardInput {
                    grade: "L80".into(),
                    outer_diameter_mm: Some(88.9),
                    min_wall_thickness_mm: Some(6.45),
                    max_wall_thickness_mm: Some(7.5),
                    length_tolerance_mm: Some(500.0),
                    source: None,
                },
            )
            .await
            .unwrap();

        fx.service
            .delete_standard(&fx.actor, view.id)
            .await
            .unwrap();
        assert!(fx.service.list_standards().await.unwrap().is_empty());

        let rows = audit_repo::page_after(&fx.pool, 0, 10).await.unwrap();
        let actions: Vec<&str> = rows.iter().map(|row| row.action.as_str()).collect();
        assert_eq!(
            actions,
            [
                actions::CREATE_STANDARD,
                actions::UPDATE_STANDARD,
                actions::DELETE_STANDARD
            ]
        );
        assert!(
            rows[2].detail.contains("\"snapshot\""),
            "删前快照必须入审计"
        );

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn standard_validation_catches_inverted_wall_range() {
        let fx = fixture("std-bad").await;
        let err = fx
            .service
            .create_standard(
                &fx.actor,
                StandardInput {
                    grade: "N80".into(),
                    outer_diameter_mm: None,
                    min_wall_thickness_mm: Some(9.0),
                    max_wall_thickness_mm: Some(7.0),
                    length_tolerance_mm: None,
                    source: None,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, CatalogError::Invalid(_)), "实际：{err:?}");
        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }
}
