//! Identity services — role/permission/department/tenant business logic.
//!
//! Follows the project convention: unit struct with static methods taking
//! `pool: &SqlitePool`, returning `Result<_, AppError>`.

use sqlx::SqlitePool;

use crate::auth::repos::{DepartmentRepo, PermissionRepo, RoleRepo, TenantRepo, UserRoleRepo};
use crate::error::AppError;
use crate::models::rbac::{Department, Permission, Role, Tenant};

pub struct IdentityService;

impl IdentityService {
    // -----------------------------------------------------------------------
    // Tenants
    // -----------------------------------------------------------------------

    pub async fn get_tenant(pool: &SqlitePool, id: i64) -> Result<Tenant, AppError> {
        TenantRepo::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Tenant not found: {}", id)))
    }

    // -----------------------------------------------------------------------
    // Permissions
    // -----------------------------------------------------------------------

    pub async fn list_permissions(pool: &SqlitePool) -> Result<Vec<Permission>, AppError> {
        PermissionRepo::list(pool).await.map_err(AppError::from)
    }

    // -----------------------------------------------------------------------
    // Roles
    // -----------------------------------------------------------------------

    pub async fn list_roles(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<Role>, AppError> {
        RoleRepo::list(pool, tenant_id).await.map_err(AppError::from)
    }

    pub async fn get_role(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<Role, AppError> {
        RoleRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Role not found: {}", id)))
    }

    pub async fn create_role(
        pool: &SqlitePool,
        tenant_id: i64,
        name: &str,
        description: Option<&str>,
    ) -> Result<Role, AppError> {
        let name = name.trim();
        if name.is_empty() || name.len() > 100 {
            return Err(AppError::Validation(
                "Role name must be 1-100 characters".into(),
            ));
        }
        if RoleRepo::find_by_name(pool, tenant_id, name).await?.is_some() {
            return Err(AppError::Validation(format!(
                "Role '{}' already exists in this tenant",
                name
            )));
        }
        RoleRepo::create(pool, tenant_id, name, description)
            .await
            .map_err(AppError::from)
    }

    pub async fn update_role(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Role, AppError> {
        let role = RoleRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Role not found: {}", id)))?;
        if let Some(new_name) = name {
            let new_name = new_name.trim();
            if new_name.is_empty() || new_name.len() > 100 {
                return Err(AppError::Validation(
                    "Role name must be 1-100 characters".into(),
                ));
            }
            if new_name != role.name
                && RoleRepo::find_by_name(pool, tenant_id, new_name)
                    .await?
                    .is_some()
            {
                return Err(AppError::Validation(format!(
                    "Role '{}' already exists in this tenant",
                    new_name
                )));
            }
        }
        RoleRepo::update(pool, tenant_id, id, name, description)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Role not found: {}", id)))
    }

    pub async fn delete_role(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<(), AppError> {
        let role = RoleRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Role not found: {}", id)))?;
        if role.is_system {
            return Err(AppError::Validation(
                "System roles cannot be deleted".into(),
            ));
        }
        RoleRepo::delete(pool, tenant_id, id)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    /// Replace the full permission set of a role. Unknown keys are rejected
    /// so a typo in the frontend never silently strips permissions.
    pub async fn set_role_permissions(
        pool: &SqlitePool,
        tenant_id: i64,
        role_id: i64,
        permission_keys: &[String],
    ) -> Result<Vec<String>, AppError> {
        let _role = RoleRepo::find_by_id(pool, tenant_id, role_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Role not found: {}", role_id)))?;

        let mut permission_ids = Vec::with_capacity(permission_keys.len());
        for key in permission_keys {
            let pid = PermissionRepo::find_id_by_key(pool, key)
                .await?
                .ok_or_else(|| AppError::Validation(format!("Unknown permission key '{}'", key)))?;
            permission_ids.push(pid);
        }
        RoleRepo::set_permissions(pool, role_id, &permission_ids)
            .await
            .map_err(AppError::from)?;
        RoleRepo::permission_keys(pool, role_id)
            .await
            .map_err(AppError::from)
    }

    pub async fn role_permission_keys(
        pool: &SqlitePool,
        tenant_id: i64,
        role_id: i64,
    ) -> Result<Vec<String>, AppError> {
        let _role = RoleRepo::find_by_id(pool, tenant_id, role_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Role not found: {}", role_id)))?;
        RoleRepo::permission_keys(pool, role_id)
            .await
            .map_err(AppError::from)
    }

    // -----------------------------------------------------------------------
    // Departments
    // -----------------------------------------------------------------------

    pub async fn list_departments(
        pool: &SqlitePool,
        tenant_id: i64,
        parent_id: Option<i64>,
    ) -> Result<Vec<Department>, AppError> {
        DepartmentRepo::list(pool, tenant_id, parent_id)
            .await
            .map_err(AppError::from)
    }

    pub async fn create_department(
        pool: &SqlitePool,
        tenant_id: i64,
        name: &str,
        parent_id: Option<i64>,
        sort_order: Option<i32>,
    ) -> Result<Department, AppError> {
        let name = name.trim();
        if name.is_empty() || name.len() > 100 {
            return Err(AppError::Validation(
                "Department name must be 1-100 characters".into(),
            ));
        }
        if let Some(pid) = parent_id {
            // Parent must exist in the same tenant.
            let parent = DepartmentRepo::find_by_id(pool, tenant_id, pid)
                .await
                .map_err(AppError::from)?;
            if parent.is_none() {
                return Err(AppError::Validation(format!(
                    "Parent department {} not found in this tenant",
                    pid
                )));
            }
        }
        DepartmentRepo::create(pool, tenant_id, name, parent_id, sort_order.unwrap_or(0))
            .await
            .map_err(AppError::from)
    }

    pub async fn update_department(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        name: Option<&str>,
        parent_id: Option<i64>,
        sort_order: Option<i32>,
    ) -> Result<Department, AppError> {
        if let Some(pid) = parent_id {
            if pid == id {
                return Err(AppError::Validation(
                    "Department cannot be its own parent".into(),
                ));
            }
            let parent = DepartmentRepo::find_by_id(pool, tenant_id, pid)
                .await
                .map_err(AppError::from)?;
            if parent.is_none() {
                return Err(AppError::Validation(format!(
                    "Parent department {} not found in this tenant",
                    pid
                )));
            }
        }
        DepartmentRepo::update(pool, tenant_id, id, name, parent_id, sort_order)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Department not found: {}", id)))
    }

    pub async fn delete_department(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<(), AppError> {
        // Block deletion when child departments exist.
        let children = DepartmentRepo::list(pool, tenant_id, Some(id))
            .await
            .map_err(AppError::from)?;
        if !children.is_empty() {
            return Err(AppError::Validation(
                "Cannot delete a department that has child departments".into(),
            ));
        }
        DepartmentRepo::delete(pool, tenant_id, id)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // User-role binding
    // -----------------------------------------------------------------------

    /// Replace a user's role set; returns the new effective permission keys.
    pub async fn assign_user_roles(
        pool: &SqlitePool,
        tenant_id: i64,
        user_id: i64,
        role_ids: &[i64],
    ) -> Result<Vec<String>, AppError> {
        let valid_roles = RoleRepo::list(pool, tenant_id).await.map_err(AppError::from)?;
        let valid_ids: Vec<i64> = valid_roles.iter().map(|r| r.id).collect();
        for rid in role_ids {
            if !valid_ids.contains(rid) {
                return Err(AppError::Validation(format!(
                    "Role {} not found in this tenant",
                    rid
                )));
            }
        }
        UserRoleRepo::assign(pool, user_id, role_ids)
            .await
            .map_err(AppError::from)?;
        Self::user_permission_keys(pool, user_id).await
    }

    pub async fn user_role_ids(pool: &SqlitePool, user_id: i64) -> Result<Vec<i64>, AppError> {
        UserRoleRepo::role_ids_for_user(pool, user_id)
            .await
            .map_err(AppError::from)
    }

    /// Effective permission keys for a user across all their roles.
    pub async fn user_permission_keys(pool: &SqlitePool, user_id: i64) -> Result<Vec<String>, AppError> {
        UserRoleRepo::permission_keys_for_user(pool, user_id)
            .await
            .map_err(AppError::from)
    }
}
