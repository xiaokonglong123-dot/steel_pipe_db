//! Integration tests for SupplierService.
//!
//! Tests CRUD, search, list, and list_active operations for suppliers.
//! All tests use an in-memory SQLite database with fresh migrations.

mod common;

use steel_pipe_db::dto::common::PaginationParams;
use steel_pipe_db::dto::supplier_dto::{
    CreateSupplierRequest, SupplierFilterParams, UpdateSupplierRequest,
};
use steel_pipe_db::services::supplier_service::SupplierService;

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Create
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_supplier_succeeds() {
    let pool = common::test_pool().await;

    let dto = CreateSupplierRequest {
        supplier_code: Some("SUP-001".into()),
        name: "Test Supplier".into(),
        contact_person: Some("John".into()),
        phone: Some("13800138000".into()),
        email: Some("supplier@test.local".into()),
        address: Some("456 Industrial Ave".into()),
        notes: None,
    };

    let supplier = SupplierService::create(&pool, &dto)
        .await
        .expect("create must succeed");

    assert_eq!(supplier.supplier_code, "SUP-001");
    assert_eq!(supplier.name, "Test Supplier");
    assert_eq!(supplier.contact_person.as_deref(), Some("John"));
    assert_eq!(supplier.phone.as_deref(), Some("13800138000"));
    assert_eq!(supplier.email.as_deref(), Some("supplier@test.local"));
    assert_eq!(supplier.address.as_deref(), Some("456 Industrial Ave"));
    assert!(supplier.is_active);
    assert!(supplier.deleted_at.is_none());
}

#[tokio::test]
async fn create_supplier_duplicate_code_fails() {
    let pool = common::test_pool().await;

    common::seed_supplier(&pool, "SUP-001", "Existing Supplier")
        .await
        .unwrap();

    let dto = CreateSupplierRequest {
        supplier_code: Some("SUP-001".into()),
        name: "Duplicate Supplier".into(),
        contact_person: None,
        phone: None,
        email: None,
        address: None,
        notes: None,
    };

    let err = SupplierService::create(&pool, &dto)
        .await
        .expect_err("duplicate code must fail");

    assert_eq!(err.error_code(), 16002);
    assert!(err.to_string().contains("already exists"));
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Update
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn update_supplier_updates_name_and_contact() {
    let pool = common::test_pool().await;

    let id = common::seed_supplier(&pool, "SUP-002", "Original Name")
        .await
        .unwrap();

    let dto = UpdateSupplierRequest {
        name: Some("Updated Name".into()),
        contact_person: Some("Jane".into()),
        phone: None,
        email: None,
        address: None,
        is_active: None,
        notes: None,
    };

    let updated = SupplierService::update(&pool, id, &dto)
        .await
        .expect("update must succeed");

    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.contact_person.as_deref(), Some("Jane"));
    // Original fields remain unchanged
    assert_eq!(updated.supplier_code, "SUP-002");
}

#[tokio::test]
async fn update_supplier_nonexistent_fails() {
    let pool = common::test_pool().await;

    let dto = UpdateSupplierRequest {
        name: Some("Ghost".into()),
        contact_person: None,
        phone: None,
        email: None,
        address: None,
        is_active: None,
        notes: None,
    };

    let err = SupplierService::update(&pool, 99999, &dto)
        .await
        .expect_err("update must fail for nonexistent supplier");

    assert_eq!(err.error_code(), 16001);
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Delete (soft delete)
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn delete_supplier_soft_deletes() {
    let pool = common::test_pool().await;

    let id = common::seed_supplier(&pool, "SUP-003", "To Delete")
        .await
        .unwrap();

    SupplierService::delete(&pool, id)
        .await
        .expect("delete must succeed");

    // Soft-deleted supplier must not be retrievable by get
    let err = SupplierService::get(&pool, id)
        .await
        .expect_err("deleted supplier must not be found");
    assert_eq!(err.error_code(), 16001);
}

#[tokio::test]
async fn delete_supplier_double_delete_fails() {
    let pool = common::test_pool().await;

    let id = common::seed_supplier(&pool, "SUP-004", "Double Delete")
        .await
        .unwrap();

    SupplierService::delete(&pool, id)
        .await
        .expect("first delete must succeed");

    let err = SupplierService::delete(&pool, id)
        .await
        .expect_err("second delete must fail");
    assert_eq!(err.error_code(), 16001);
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Get
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn get_supplier_succeeds() {
    let pool = common::test_pool().await;

    let id = common::seed_supplier(&pool, "SUP-005", "Fetch Me")
        .await
        .unwrap();

    let supplier = SupplierService::get(&pool, id)
        .await
        .expect("get must succeed");

    assert_eq!(supplier.supplier_code, "SUP-005");
    assert_eq!(supplier.name, "Fetch Me");
}

#[tokio::test]
async fn get_supplier_nonexistent_fails() {
    let pool = common::test_pool().await;

    let err = SupplierService::get(&pool, 99999)
        .await
        .expect_err("get must fail for nonexistent supplier");
    assert_eq!(err.error_code(), 16001);
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// List (pagination + filters)
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn list_suppliers_pagination() {
    let pool = common::test_pool().await;

    // Seed 3 suppliers
    common::seed_supplier(&pool, "SUP-010", "Alpha").await.unwrap();
    common::seed_supplier(&pool, "SUP-011", "Beta").await.unwrap();
    common::seed_supplier(&pool, "SUP-012", "Gamma").await.unwrap();

    let filter = SupplierFilterParams {
        q: None,
        is_active: None,
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };
    let params = PaginationParams {
        page: Some(1),
        page_size: Some(2),
        sort_by: None,
        sort_order: None,
    };

    let (items, total) = SupplierService::list(&pool, &filter, &params)
        .await
        .expect("list must succeed");

    assert_eq!(items.len(), 2, "page_size=2 should return 2 items");
    assert_eq!(total, 3, "total should be 3");
}

#[tokio::test]
async fn list_suppliers_search_by_code() {
    let pool = common::test_pool().await;

    common::seed_supplier(&pool, "SUP-020", "Target Corp")
        .await
        .unwrap();
    common::seed_supplier(&pool, "SUP-021", "Other Inc")
        .await
        .unwrap();

    let filter = SupplierFilterParams {
        q: Some("SUP-020".into()),
        is_active: None,
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };
    let params = PaginationParams {
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };

    let (items, total) = SupplierService::list(&pool, &filter, &params)
        .await
        .expect("list must succeed");

    assert_eq!(total, 1);
    assert_eq!(items[0].supplier_code, "SUP-020");
}

#[tokio::test]
async fn list_suppliers_search_by_name() {
    let pool = common::test_pool().await;

    common::seed_supplier(&pool, "SUP-030", "Searchable Corp")
        .await
        .unwrap();
    common::seed_supplier(&pool, "SUP-031", "Other Inc")
        .await
        .unwrap();

    let filter = SupplierFilterParams {
        q: Some("Searchable".into()),
        is_active: None,
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };
    let params = PaginationParams {
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };

    let (items, total) = SupplierService::list(&pool, &filter, &params)
        .await
        .expect("list must succeed");

    assert_eq!(total, 1);
    assert_eq!(items[0].name, "Searchable Corp");
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Search (by keyword)
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn search_suppliers_by_keyword() {
    let pool = common::test_pool().await;

    common::seed_supplier(&pool, "SUP-040", "Found Co").await.unwrap();
    common::seed_supplier(&pool, "SUP-041", "Hidden Co").await.unwrap();

    let results = SupplierService::search(&pool, "Found")
        .await
        .expect("search must succeed");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Found Co");
}

#[tokio::test]
async fn search_suppliers_empty_query_fails() {
    let pool = common::test_pool().await;

    let err = SupplierService::search(&pool, "")
        .await
        .expect_err("empty search must fail");

    assert_eq!(err.error_code(), 10002);
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// List Active
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn list_active_suppliers_returns_only_active() {
    let pool = common::test_pool().await;

    // Seed two active suppliers
    let id1 = common::seed_supplier(&pool, "SUP-050", "Active One")
        .await
        .unwrap();
    common::seed_supplier(&pool, "SUP-051", "Active Two")
        .await
        .unwrap();

    // Soft-delete the first one only
    SupplierService::delete(&pool, id1).await.unwrap();

    let active = SupplierService::list_active(&pool)
        .await
        .expect("list_active must succeed");

    assert_eq!(active.len(), 1, "only the non-deleted supplier should appear");
    assert_eq!(active[0].supplier_code, "SUP-051");
}
