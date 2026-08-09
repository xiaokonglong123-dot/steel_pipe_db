//! RBAC repositories — pure SQL over the auth schema tables (tenants,
//! departments, roles, permissions, role_permissions, user_roles).
//!
//! Follows the project convention: unit structs with static methods taking
//! `pool: &SqlitePool`. Soft-delete aware via `deleted_at IS NULL`.

use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};

use crate::models::rbac::{Department, Permission, Role, Tenant};

/// Tenant repository.
pub struct TenantRepo;

impl TenantRepo {
    pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Tenant>, sqlx::Error> {
        sqlx::query_as::<_, Tenant>(
            "SELECT id, code, name, is_active, created_at, updated_at, deleted_at \
             FROM tenants WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

/// Department repository.
pub struct DepartmentRepo;

impl DepartmentRepo {
    pub async fn find_by_id(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
    ) -> Result<Option<Department>, sqlx::Error> {
        sqlx::query_as::<_, Department>(
            "SELECT id, tenant_id, name, parent_id, path, sort_order, created_at, updated_at, deleted_at \
             FROM departments WHERE tenant_id = ? AND id = ? AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn list(
        pool: &SqlitePool,
        tenant_id: i64,
        parent_id: Option<i64>,
    ) -> Result<Vec<Department>, sqlx::Error> {
        sqlx::query_as::<_, Department>(
            "SELECT id, tenant_id, name, parent_id, path, sort_order, created_at, updated_at, deleted_at \
             FROM departments WHERE tenant_id = ? AND deleted_at IS NULL \
             AND (? IS NULL OR parent_id = ?) ORDER BY sort_order, id",
        )
        .bind(tenant_id)
        .bind(parent_id)
        .bind(parent_id)
        .fetch_all(pool)
        .await
    }

    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        name: &str,
        parent_id: Option<i64>,
        sort_order: i32,
    ) -> Result<Department, sqlx::Error> {
        sqlx::query_as::<_, Department>(
            "INSERT INTO departments (tenant_id, name, parent_id, path, sort_order) \
             VALUES (?, ?, ?, \
                 CASE WHEN ? IS NULL THEN '/' || ? ELSE \
                     (SELECT path FROM departments WHERE id = ?) || '/' || ? END, \
                 ?) \
             RETURNING id, tenant_id, name, parent_id, path, sort_order, created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(name)
        .bind(parent_id)
        .bind(parent_id)
        .bind(name)
        .bind(parent_id)
        .bind(name)
        .bind(sort_order)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        name: Option<&str>,
        parent_id: Option<i64>,
        sort_order: Option<i32>,
    ) -> Result<Option<Department>, sqlx::Error> {
        sqlx::query_as::<_, Department>(
            "UPDATE departments SET \
                name = COALESCE(?, name), \
                parent_id = ?, \
                sort_order = COALESCE(?, sort_order), \
                updated_at = datetime('now') \
             WHERE id = ? AND tenant_id = ? AND deleted_at IS NULL \
             RETURNING id, tenant_id, name, parent_id, path, sort_order, created_at, updated_at, deleted_at",
        )
        .bind(name)
        .bind(parent_id)
        .bind(sort_order)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<bool, sqlx::Error> {
        let res = sqlx::query(
            "UPDATE departments SET deleted_at = datetime('now') \
             WHERE id = ? AND tenant_id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .bind(tenant_id)
        .execute(pool)
        .await?;
        Ok(res.rows_affected() > 0)
    }
}

/// Role repository.
pub struct RoleRepo;

impl RoleRepo {
    pub async fn list(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<Role>, sqlx::Error> {
        sqlx::query_as::<_, Role>(
            "SELECT id, tenant_id, name, description, is_system, created_at, updated_at, deleted_at \
             FROM roles WHERE tenant_id = ? AND deleted_at IS NULL ORDER BY id",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<Option<Role>, sqlx::Error> {
        sqlx::query_as::<_, Role>(
            "SELECT id, tenant_id, name, description, is_system, created_at, updated_at, deleted_at \
             FROM roles WHERE id = ? AND tenant_id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_name(
        pool: &SqlitePool,
        tenant_id: i64,
        name: &str,
    ) -> Result<Option<Role>, sqlx::Error> {
        sqlx::query_as::<_, Role>(
            "SELECT id, tenant_id, name, description, is_system, created_at, updated_at, deleted_at \
             FROM roles WHERE tenant_id = ? AND name = ? AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(name)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        name: &str,
        description: Option<&str>,
    ) -> Result<Role, sqlx::Error> {
        sqlx::query_as::<_, Role>(
            "INSERT INTO roles (tenant_id, name, description) VALUES (?, ?, ?) \
             RETURNING id, tenant_id, name, description, is_system, created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(name)
        .bind(description)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Option<Role>, sqlx::Error> {
        sqlx::query_as::<_, Role>(
            "UPDATE roles SET \
                name = COALESCE(?, name), \
                description = COALESCE(?, description), \
                updated_at = datetime('now') \
             WHERE id = ? AND tenant_id = ? AND deleted_at IS NULL \
             RETURNING id, tenant_id, name, description, is_system, created_at, updated_at, deleted_at",
        )
        .bind(name)
        .bind(description)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<bool, sqlx::Error> {
        let res = sqlx::query(
            "UPDATE roles SET deleted_at = datetime('now') \
             WHERE id = ? AND tenant_id = ? AND deleted_at IS NULL AND is_system = 0",
        )
        .bind(id)
        .bind(tenant_id)
        .execute(pool)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    /// Permission keys granted to a role, in stable order.
    pub async fn permission_keys(pool: &SqlitePool, role_id: i64) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT p.key FROM permissions p \
             JOIN role_permissions rp ON rp.permission_id = p.id \
             WHERE rp.role_id = ? ORDER BY p.key",
        )
        .bind(role_id)
        .fetch_all(pool)
        .await?;
        Ok(rows.iter().map(|r| r.get::<String, _>(0)).collect())
    }

    /// Replace the full permission set of a role.
    pub async fn set_permissions(
        pool: &SqlitePool,
        role_id: i64,
        permission_ids: &[i64],
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        sqlx::query("DELETE FROM role_permissions WHERE role_id = ?")
            .bind(role_id)
            .execute(&mut *tx)
            .await?;
        for pid in permission_ids {
            sqlx::query(
                "INSERT INTO role_permissions (role_id, permission_id) VALUES (?, ?) \
                 ON CONFLICT DO NOTHING",
            )
            .bind(role_id)
            .bind(pid)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await
    }
}

/// Permission dictionary repository.
pub struct PermissionRepo;

impl PermissionRepo {
    pub async fn list(pool: &SqlitePool) -> Result<Vec<Permission>, sqlx::Error> {
        sqlx::query_as::<_, Permission>("SELECT id, key, description FROM permissions ORDER BY key")
            .fetch_all(pool)
            .await
    }

    pub async fn find_id_by_key(pool: &SqlitePool, key: &str) -> Result<Option<i64>, sqlx::Error> {
        sqlx::query_scalar("SELECT id FROM permissions WHERE key = ?")
            .bind(key)
            .fetch_optional(pool)
            .await
    }
}

/// User-role association repository.
pub struct UserRoleRepo;

impl UserRoleRepo {
    pub async fn assign(
        pool: &SqlitePool,
        user_id: i64,
        role_ids: &[i64],
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        sqlx::query("DELETE FROM user_roles WHERE user_id = ?")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        for rid in role_ids {
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_id) VALUES (?, ?) ON CONFLICT DO NOTHING",
            )
            .bind(user_id)
            .bind(rid)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await
    }

    pub async fn role_ids_for_user(pool: &SqlitePool, user_id: i64) -> Result<Vec<i64>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT role_id FROM user_roles WHERE user_id = ? \
             AND role_id IN (SELECT id FROM roles WHERE deleted_at IS NULL) ORDER BY role_id",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        Ok(rows.iter().map(|r| r.get::<i64, _>(0)).collect())
    }

    /// All permission keys granted to a user across their roles (deduplicated).
    pub async fn permission_keys_for_user(
        pool: &SqlitePool,
        user_id: i64,
    ) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT DISTINCT p.key FROM permissions p \
             JOIN role_permissions rp ON rp.permission_id = p.id \
             JOIN user_roles ur ON ur.role_id = rp.role_id \
             WHERE ur.user_id = ? \
               AND ur.role_id IN (SELECT id FROM roles WHERE deleted_at IS NULL) \
             ORDER BY p.key",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        Ok(rows.iter().map(|r| r.get::<String, _>(0)).collect())
    }
}

/// Local helper to snapshot a timestamp used for "changed since" semantics.
#[allow(dead_code)]
pub fn now() -> DateTime<Utc> {
    Utc::now()
}
