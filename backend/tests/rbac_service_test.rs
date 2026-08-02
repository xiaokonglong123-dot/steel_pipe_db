//! RBAC integration tests — roles, permissions, departments, user-role binding.
//!
//! These exercise the IdentityService against a real (per-test schema)
//! PostgreSQL instance. Migration 022/023/024 seed the permission
//! dictionary and the four built-in roles.

mod common;

use steel_pipe_db::auth::services::IdentityService;

#[tokio::test]
async fn list_permissions_returns_dictionary() {
    let pool = common::test_pool().await;
    let perms = IdentityService::list_permissions(&pool).await.unwrap();
    assert!(perms.len() >= 20, "expected >=20 seeded permissions, got {}", perms.len());
    assert!(perms.iter().any(|p| p.key == "pipe.read"));
    assert!(perms.iter().any(|p| p.key == "system.admin"));
}

#[tokio::test]
async fn list_roles_returns_seeded_roles() {
    let pool = common::test_pool().await;
    let roles = IdentityService::list_roles(&pool, 1).await.unwrap();
    assert_eq!(roles.len(), 4, "expected 4 seeded roles");
    let admin = roles.iter().find(|r| r.name == "admin").unwrap();
    assert!(admin.is_system, "admin role must be system");
}

#[tokio::test]
async fn create_role_then_duplicate_rejected() {
    let pool = common::test_pool().await;
    let role = IdentityService::create_role(&pool, 1, "supervisor", Some("车间主管"))
        .await
        .unwrap();
    assert_eq!(role.name, "supervisor");
    assert!(!role.is_system);

    // Duplicate name in the same tenant must be rejected.
    let dup = IdentityService::create_role(&pool, 1, "supervisor", None).await;
    assert!(dup.is_err(), "duplicate role name must be rejected");
}

#[tokio::test]
async fn system_role_cannot_be_deleted() {
    let pool = common::test_pool().await;
    let err = IdentityService::delete_role(&pool, 1, 1).await; // id=1 is admin
    assert!(err.is_err(), "deleting a system role must fail");
}

#[tokio::test]
async fn set_role_permissions_replaces_and_returns_keys() {
    let pool = common::test_pool().await;
    let role = IdentityService::create_role(&pool, 1, "inspector", None)
        .await
        .unwrap();

    let keys = IdentityService::set_role_permissions(
        &pool,
        1,
        role.id,
        &["pipe.read".to_string(), "quality.read".to_string()],
    )
    .await
    .unwrap();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"pipe.read".to_string()));

    // Unknown permission keys are rejected (typo safety).
    let bad = IdentityService::set_role_permissions(
        &pool,
        1,
        role.id,
        &["pipe.nonexistent".to_string()],
    )
    .await;
    assert!(bad.is_err(), "unknown permission key must be rejected");
}

#[tokio::test]
async fn role_permission_keys_reflects_assignment() {
    let pool = common::test_pool().await;
    let role = IdentityService::create_role(&pool, 1, "clerk", None)
        .await
        .unwrap();
    IdentityService::set_role_permissions(
        &pool,
        1,
        role.id,
        &["inventory.inbound".to_string(), "inventory.outbound".to_string()],
    )
    .await
    .unwrap();

    let keys = IdentityService::role_permission_keys(&pool, 1, role.id).await.unwrap();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"inventory.outbound".to_string()));
}

#[tokio::test]
async fn user_permission_keys_aggregates_across_roles() {
    let pool = common::test_pool().await;
    let user_id = common::seed_user(&pool, "agguser", "warehouse").await.unwrap();
    let admin_role_id = 1; // admin role from seed

    // Assign admin role to the user → effective permissions should now
    // include system.admin (admin role holds every permission).
    let keys = IdentityService::assign_user_roles(&pool, 1, user_id, &[admin_role_id])
        .await
        .unwrap();
    assert!(keys.contains(&"system.admin".to_string()), "admin role must grant system.admin");

    let ids = IdentityService::user_role_ids(&pool, user_id).await.unwrap();
    assert!(ids.contains(&admin_role_id));
}

#[tokio::test]
async fn assign_user_roles_rejects_cross_tenant_role() {
    let pool = common::test_pool().await;
    let user_id = common::seed_user(&pool, "crossuser", "sales").await.unwrap();
    // Role id 999 does not exist in tenant 1.
    let err = IdentityService::assign_user_roles(&pool, 1, user_id, &[999]).await;
    assert!(err.is_err(), "role outside the tenant must be rejected");
}

#[tokio::test]
async fn departments_crud_and_children_block() {
    let pool = common::test_pool().await;
    let parent = IdentityService::create_department(&pool, 1, "生产部", None, None)
        .await
        .unwrap();
    let child = IdentityService::create_department(&pool, 1, "一车间", Some(parent.id), None)
        .await
        .unwrap();
    assert_eq!(child.parent_id, Some(parent.id));

    // Deleting a parent with children must be blocked.
    let err = IdentityService::delete_department(&pool, 1, parent.id).await;
    assert!(err.is_err(), "deleting a department with children must fail");

    // Deleting the child works.
    IdentityService::delete_department(&pool, 1, child.id).await.unwrap();
    // Now the parent is deletable.
    IdentityService::delete_department(&pool, 1, parent.id).await.unwrap();
}

#[tokio::test]
async fn department_cannot_be_own_parent() {
    let pool = common::test_pool().await;
    let dept = IdentityService::create_department(&pool, 1, "质检部", None, None)
        .await
        .unwrap();
    let err = IdentityService::update_department(&pool, 1, dept.id, None, Some(dept.id), None).await;
    assert!(err.is_err(), "self-parenting must be rejected");
}
