//! Integration tests for CustomerService.
//!
//! Tests CRUD, search, list, and list_active operations for customers.
//! All tests use an in-memory SQLite database with fresh migrations.

mod common;

use steel_pipe_db::dto::common::PaginationParams;
use steel_pipe_db::dto::customer_dto::{
    CreateCustomerRequest, CustomerFilterParams, UpdateCustomerRequest,
};
use steel_pipe_db::services::customer_service::CustomerService;

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Create
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_customer_succeeds() {
    let pool = common::test_pool().await;

    let dto = CreateCustomerRequest {
        customer_code: Some("CUS-001".into()),
        name: "Test Customer".into(),
        contact_person: Some("John".into()),
        phone: Some("13800138001".into()),
        email: Some("customer@test.local".into()),
        address: Some("123 Main St".into()),
        notes: None,
    };

    let customer = CustomerService::create(&pool, &dto)
        .await
        .expect("create must succeed");

    assert_eq!(customer.customer_code, "CUS-001");
    assert_eq!(customer.name, "Test Customer");
    assert_eq!(customer.contact_person.as_deref(), Some("John"));
    assert_eq!(customer.phone.as_deref(), Some("13800138001"));
    assert_eq!(customer.email.as_deref(), Some("customer@test.local"));
    assert_eq!(customer.address.as_deref(), Some("123 Main St"));
    assert!(customer.is_active);
    assert!(customer.deleted_at.is_none());
}

#[tokio::test]
async fn create_customer_duplicate_code_fails() {
    let pool = common::test_pool().await;

    common::seed_customer(&pool, "CUS-001", "Existing Customer")
        .await
        .unwrap();

    let dto = CreateCustomerRequest {
        customer_code: Some("CUS-001".into()),
        name: "Duplicate Customer".into(),
        contact_person: None,
        phone: None,
        email: None,
        address: None,
        notes: None,
    };

    let err = CustomerService::create(&pool, &dto)
        .await
        .expect_err("duplicate code must fail");

    assert_eq!(err.error_code(), 17002);
    assert!(err.to_string().contains("already exists"));
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Update
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn update_customer_updates_name_and_contact() {
    let pool = common::test_pool().await;

    let id = common::seed_customer(&pool, "CUS-002", "Original Name")
        .await
        .unwrap();

    let dto = UpdateCustomerRequest {
        name: Some("Updated Name".into()),
        contact_person: Some("Jane".into()),
        phone: None,
        email: None,
        address: None,
        is_active: None,
        notes: None,
    };

    let updated = CustomerService::update(&pool, id, &dto)
        .await
        .expect("update must succeed");

    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.contact_person.as_deref(), Some("Jane"));
    // Original fields remain unchanged
    assert_eq!(updated.customer_code, "CUS-002");
}

#[tokio::test]
async fn update_customer_nonexistent_fails() {
    let pool = common::test_pool().await;

    let dto = UpdateCustomerRequest {
        name: Some("Ghost".into()),
        contact_person: None,
        phone: None,
        email: None,
        address: None,
        is_active: None,
        notes: None,
    };

    let err = CustomerService::update(&pool, 99999, &dto)
        .await
        .expect_err("update must fail for nonexistent customer");

    assert_eq!(err.error_code(), 17001);
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Delete (soft delete)
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn delete_customer_soft_deletes() {
    let pool = common::test_pool().await;

    let id = common::seed_customer(&pool, "CUS-003", "To Delete")
        .await
        .unwrap();

    CustomerService::delete(&pool, id)
        .await
        .expect("delete must succeed");

    // Soft-deleted customer must not be retrievable by get
    let err = CustomerService::get(&pool, id)
        .await
        .expect_err("deleted customer must not be found");
    assert_eq!(err.error_code(), 17001);
}

#[tokio::test]
async fn delete_customer_double_delete_fails() {
    let pool = common::test_pool().await;

    let id = common::seed_customer(&pool, "CUS-004", "Double Delete")
        .await
        .unwrap();

    CustomerService::delete(&pool, id)
        .await
        .expect("first delete must succeed");

    let err = CustomerService::delete(&pool, id)
        .await
        .expect_err("second delete must fail");
    assert_eq!(err.error_code(), 17001);
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Get
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn get_customer_succeeds() {
    let pool = common::test_pool().await;

    let id = common::seed_customer(&pool, "CUS-005", "Fetch Me")
        .await
        .unwrap();

    let customer = CustomerService::get(&pool, id)
        .await
        .expect("get must succeed");

    assert_eq!(customer.customer_code, "CUS-005");
    assert_eq!(customer.name, "Fetch Me");
}

#[tokio::test]
async fn get_customer_nonexistent_fails() {
    let pool = common::test_pool().await;

    let err = CustomerService::get(&pool, 99999)
        .await
        .expect_err("get must fail for nonexistent customer");
    assert_eq!(err.error_code(), 17001);
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// List (pagination + filters)
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn list_customers_pagination() {
    let pool = common::test_pool().await;

    // Seed 3 customers
    common::seed_customer(&pool, "CUS-010", "Alpha").await.unwrap();
    common::seed_customer(&pool, "CUS-011", "Beta").await.unwrap();
    common::seed_customer(&pool, "CUS-012", "Gamma").await.unwrap();

    let filter = CustomerFilterParams {
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

    let (items, total) = CustomerService::list(&pool, &filter, &params)
        .await
        .expect("list must succeed");

    assert_eq!(items.len(), 2, "page_size=2 should return 2 items");
    assert_eq!(total, 3, "total should be 3");
}

#[tokio::test]
async fn list_customers_search_by_code() {
    let pool = common::test_pool().await;

    common::seed_customer(&pool, "CUS-020", "Target Corp")
        .await
        .unwrap();
    common::seed_customer(&pool, "CUS-021", "Other Inc")
        .await
        .unwrap();

    let filter = CustomerFilterParams {
        q: Some("CUS-020".into()),
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

    let (items, total) = CustomerService::list(&pool, &filter, &params)
        .await
        .expect("list must succeed");

    assert_eq!(total, 1);
    assert_eq!(items[0].customer_code, "CUS-020");
}

#[tokio::test]
async fn list_customers_search_by_name() {
    let pool = common::test_pool().await;

    common::seed_customer(&pool, "CUS-030", "Searchable Corp")
        .await
        .unwrap();
    common::seed_customer(&pool, "CUS-031", "Other Inc")
        .await
        .unwrap();

    let filter = CustomerFilterParams {
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

    let (items, total) = CustomerService::list(&pool, &filter, &params)
        .await
        .expect("list must succeed");

    assert_eq!(total, 1);
    assert_eq!(items[0].name, "Searchable Corp");
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Search (by keyword)
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn search_customers_by_keyword() {
    let pool = common::test_pool().await;

    common::seed_customer(&pool, "CUS-040", "Found Co").await.unwrap();
    common::seed_customer(&pool, "CUS-041", "Hidden Co").await.unwrap();

    let results = CustomerService::search(&pool, "Found")
        .await
        .expect("search must succeed");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Found Co");
}

#[tokio::test]
async fn search_customers_empty_query_fails() {
    let pool = common::test_pool().await;

    let err = CustomerService::search(&pool, "")
        .await
        .expect_err("empty search must fail");

    assert_eq!(err.error_code(), 10002);
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// List Active
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn list_active_customers_returns_only_active() {
    let pool = common::test_pool().await;

    // Seed two active customers
    let id1 = common::seed_customer(&pool, "CUS-050", "Active One")
        .await
        .unwrap();
    common::seed_customer(&pool, "CUS-051", "Active Two")
        .await
        .unwrap();

    // Soft-delete the first one only
    CustomerService::delete(&pool, id1).await.unwrap();

    let active = CustomerService::list_active(&pool)
        .await
        .expect("list_active must succeed");

    assert_eq!(active.len(), 1, "only the non-deleted customer should appear");
    assert_eq!(active[0].customer_code, "CUS-051");
}
