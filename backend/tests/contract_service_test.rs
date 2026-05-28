//! Integration tests for ContractService.
//!
//! Covers:
//! - Contract CRUD (create, read, update, soft delete, listing)
//! - Status lifecycle transitions (draft → active → completed)
//! - Line items CRUD
//! - Payment milestones CRUD
//!
//! All tests use an in-memory SQLite database with fresh migrations.

mod common;

use steel_pipe_db::dto::common::PaginationParams;
use steel_pipe_db::dto::contract_dto::{
    ContractFilterParams, CreateContractItemRequest, CreateContractRequest, CreatePaymentRequest,
    UpdateContractItemRequest, UpdateContractRequest, UpdatePaymentRequest,
};
use steel_pipe_db::services::contract_service::ContractService;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// create_contract
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_contract_sales_success() {
    let pool = common::test_pool().await;

    let dto = CreateContractRequest {
        contract_type: "sales".into(),
        title: "Sales Contract A".into(),
        party_a: "Seller Corp".into(),
        party_b: "Buyer Corp".into(),
        sign_date: Some("2025-01-15".into()),
        start_date: Some("2025-02-01".into()),
        end_date: Some("2025-12-31".into()),
        notes: Some("test sales contract".into()),
        items: vec![CreateContractItemRequest {
            pipe_type: "seamless".into(),
            grade: "L80".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 100,
            unit_price: Some(500.0),
            notes: None,
        }],
    };

    let detail = ContractService::create_contract(&pool, &dto)
        .await
        .expect("create_contract must succeed");

    assert!(detail.contract.id > 0);
    assert!(detail.contract.contract_no.starts_with("CT-SAL-"));
    assert_eq!(detail.contract.contract_type, "sales");
    assert_eq!(detail.contract.title, "Sales Contract A");
    assert_eq!(detail.contract.party_a, "Seller Corp");
    assert_eq!(detail.contract.party_b, "Buyer Corp");
    assert_eq!(detail.contract.status, "draft");
    assert_eq!(detail.contract.notes.as_deref(), Some("test sales contract"));
    assert_eq!(detail.items.len(), 1);
    assert_eq!(detail.items[0].quantity, 100);
    assert_eq!(detail.items[0].grade, "L80");
    assert!(detail.contract.total_amount.unwrap_or(0.0) > 0.0);
    assert!(detail.contract.deleted_at.is_none());
}

#[tokio::test]
async fn create_contract_purchase_success() {
    let pool = common::test_pool().await;

    let dto = CreateContractRequest {
        contract_type: "purchase".into(),
        title: "Purchase Contract B".into(),
        party_a: "Buyer Inc".into(),
        party_b: "Supplier Inc".into(),
        sign_date: Some("2025-03-01".into()),
        start_date: None,
        end_date: None,
        notes: None,
        items: vec![
            CreateContractItemRequest {
                pipe_type: "seamless".into(),
                grade: "J55".into(),
                od: 139.7,
                wt: 7.72,
                quantity: 50,
                unit_price: Some(450.0),
                notes: None,
            },
            CreateContractItemRequest {
                pipe_type: "screen".into(),
                grade: "N80".into(),
                od: 177.8,
                wt: 9.19,
                quantity: 20,
                unit_price: Some(600.0),
                notes: Some("premium screen".into()),
            },
        ],
    };

    let detail = ContractService::create_contract(&pool, &dto)
        .await
        .expect("create_contract must succeed");

    assert!(detail.contract.contract_no.starts_with("CT-PUR-"));
    assert_eq!(detail.contract.contract_type, "purchase");
    assert_eq!(detail.contract.title, "Purchase Contract B");
    assert_eq!(detail.items.len(), 2);
    // Verify total_amount was recalculated
    let expected_total = 50.0 * 450.0 + 20.0 * 600.0;
    assert_eq!(detail.contract.total_amount, Some(expected_total));
}

#[tokio::test]
async fn create_contract_invalid_type_rejected() {
    let pool = common::test_pool().await;

    let dto = CreateContractRequest {
        contract_type: "invalid_type".into(),
        title: "Bad Contract".into(),
        party_a: "A".into(),
        party_b: "B".into(),
        sign_date: None,
        start_date: None,
        end_date: None,
        notes: None,
        items: vec![CreateContractItemRequest {
            pipe_type: "seamless".into(),
            grade: "L80".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 10,
            unit_price: None,
            notes: None,
        }],
    };

    let err = ContractService::create_contract(&pool, &dto)
        .await
        .expect_err("must reject invalid contract type");
    assert!(err.to_string().contains("Invalid contract type"));
}

#[tokio::test]
async fn create_contract_empty_items_rejected() {
    let pool = common::test_pool().await;

    let dto = CreateContractRequest {
        contract_type: "sales".into(),
        title: "No Items Contract".into(),
        party_a: "A".into(),
        party_b: "B".into(),
        sign_date: None,
        start_date: None,
        end_date: None,
        notes: None,
        items: vec![],
    };

    let err = ContractService::create_contract(&pool, &dto)
        .await
        .expect_err("must reject empty items");
    assert!(err.to_string().contains("at least one item"));
}

#[tokio::test]
async fn create_contract_zero_quantity_rejected() {
    let pool = common::test_pool().await;

    let dto = CreateContractRequest {
        contract_type: "sales".into(),
        title: "Zero Quantity".into(),
        party_a: "A".into(),
        party_b: "B".into(),
        sign_date: None,
        start_date: None,
        end_date: None,
        notes: None,
        items: vec![CreateContractItemRequest {
            pipe_type: "seamless".into(),
            grade: "L80".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 0,
            unit_price: None,
            notes: None,
        }],
    };

    let err = ContractService::create_contract(&pool, &dto)
        .await
        .expect_err("must reject zero quantity");
    assert!(err.to_string().contains("quantity must be positive"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// update_contract
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn update_contract_title_and_status() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-UPD-001", "sales", "Test Title", "draft")
        .await
        .unwrap();

    let update = UpdateContractRequest {
        title: Some("Updated Title".into()),
        party_a: Some("Updated Party A".into()),
        party_b: None,
        sign_date: Some("2025-06-01".into()),
        start_date: None,
        end_date: None,
        notes: Some("updated notes".into()),
    };

    let updated = ContractService::update_contract(&pool, contract_id, &update)
        .await
        .expect("update_contract must succeed");

    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.party_a, "Updated Party A");
    assert_eq!(updated.notes.as_deref(), Some("updated notes"));
    assert_eq!(updated.sign_date.as_deref(), Some("2025-06-01"));
}

#[tokio::test]
async fn update_contract_nonexistent_fails() {
    let pool = common::test_pool().await;

    let update = UpdateContractRequest {
        title: Some("Ghost".into()),
        party_a: None,
        party_b: None,
        sign_date: None,
        start_date: None,
        end_date: None,
        notes: None,
    };

    let err = ContractService::update_contract(&pool, 99999, &update)
        .await
        .expect_err("must fail for non-existent contract");
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn update_contract_active_status_rejected() {
    let pool = common::test_pool().await;

    // Seed an active contract (need sign_date for valid activation)
    let contract_id = common::seed_contract(&pool, "CT-UPD-002", "sales", "Active", "active")
        .await
        .unwrap();

    let update = UpdateContractRequest {
        title: Some("Should Not Update".into()),
        party_a: None,
        party_b: None,
        sign_date: None,
        start_date: None,
        end_date: None,
        notes: None,
    };

    let err = ContractService::update_contract(&pool, contract_id, &update)
        .await
        .expect_err("must reject update on active contract");
    assert!(err.to_string().contains("Cannot modify"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// delete_contract (soft delete)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn delete_contract_soft_deletes() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-DEL-001", "sales", "Delete Me", "draft")
        .await
        .unwrap();

    ContractService::delete_contract(&pool, contract_id)
        .await
        .expect("delete_contract must succeed");

    // Verify soft-deleted: get_contract_detail should fail
    let err = ContractService::get_contract_detail(&pool, contract_id)
        .await
        .expect_err("deleted contract must not be found");
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn delete_contract_nonexistent_fails() {
    let pool = common::test_pool().await;

    let err = ContractService::delete_contract(&pool, 99999)
        .await
        .expect_err("must fail for non-existent contract");
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn delete_contract_active_status_rejected() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-DEL-002", "purchase", "Active Del", "active")
        .await
        .unwrap();

    let err = ContractService::delete_contract(&pool, contract_id)
        .await
        .expect_err("must reject delete on active contract");
    assert!(err.to_string().contains("Cannot delete"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// get_contract_detail
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn get_contract_detail_with_items_and_payments() {
    let pool = common::test_pool().await;

    let dto = CreateContractRequest {
        contract_type: "sales".into(),
        title: "Detail Test".into(),
        party_a: "A".into(),
        party_b: "B".into(),
        sign_date: Some("2025-05-01".into()),
        start_date: None,
        end_date: None,
        notes: None,
        items: vec![
            CreateContractItemRequest {
                pipe_type: "seamless".into(),
                grade: "L80".into(),
                od: 177.8,
                wt: 9.19,
                quantity: 30,
                unit_price: Some(550.0),
                notes: None,
            },
            CreateContractItemRequest {
                pipe_type: "screen".into(),
                grade: "N80".into(),
                od: 177.8,
                wt: 9.19,
                quantity: 10,
                unit_price: Some(700.0),
                notes: Some("special".into()),
            },
        ],
    };

    let detail = ContractService::create_contract(&pool, &dto)
        .await
        .expect("create must succeed");
    let contract_id = detail.contract.id;

    // Add a payment milestone
    let pay_dto = CreatePaymentRequest {
        due_date: "2025-06-15".into(),
        amount: 10000.0,
        payment_type: "deposit".into(),
        notes: Some("initial deposit".into()),
    };
    ContractService::add_payment(&pool, contract_id, &pay_dto)
        .await
        .expect("add_payment must succeed");

    // Fetch detail
    let fetched = ContractService::get_contract_detail(&pool, contract_id)
        .await
        .expect("get_contract_detail must succeed");

    assert_eq!(fetched.contract.id, contract_id);
    assert_eq!(fetched.contract.title, "Detail Test");
    assert_eq!(fetched.items.len(), 2);
    assert_eq!(fetched.payments.len(), 1);
    assert_eq!(fetched.payments[0].payment_type, "deposit");
    assert_eq!(fetched.payments[0].amount, 10000.0);
}

#[tokio::test]
async fn get_contract_detail_nonexistent_fails() {
    let pool = common::test_pool().await;

    let err = ContractService::get_contract_detail(&pool, 99999)
        .await
        .expect_err("must fail for non-existent contract");
    assert!(err.to_string().contains("not found"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// list_contracts
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn list_contracts_pagination() {
    let pool = common::test_pool().await;

    common::seed_contract(&pool, "CT-LST-001", "sales", "Contract 1", "draft")
        .await
        .unwrap();
    common::seed_contract(&pool, "CT-LST-002", "purchase", "Contract 2", "draft")
        .await
        .unwrap();

    let filter = ContractFilterParams {
        q: None,
        contract_type: None,
        status: None,
        date_from: None,
        date_to: None,
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

    let (items, total) = ContractService::list_contracts(&pool, &filter, &params)
        .await
        .expect("list_contracts must succeed");
    assert_eq!(total, 2);
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn list_contracts_filter_by_status() {
    let pool = common::test_pool().await;

    common::seed_contract(&pool, "CT-LST-010", "sales", "Draft Contract", "draft")
        .await
        .unwrap();
    common::seed_contract(&pool, "CT-LST-011", "sales", "Active Contract", "active")
        .await
        .unwrap();

    let filter = ContractFilterParams {
        q: None,
        contract_type: None,
        status: Some("draft".into()),
        date_from: None,
        date_to: None,
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

    let (items, total) = ContractService::list_contracts(&pool, &filter, &params)
        .await
        .expect("list_contracts must succeed");
    assert_eq!(total, 1);
    assert_eq!(items[0].status, "draft");
}

#[tokio::test]
async fn list_contracts_filter_by_type() {
    let pool = common::test_pool().await;

    common::seed_contract(&pool, "CT-LST-020", "sales", "Sales Contract", "draft")
        .await
        .unwrap();
    common::seed_contract(&pool, "CT-LST-021", "purchase", "Purchase Contract", "draft")
        .await
        .unwrap();

    let filter = ContractFilterParams {
        q: None,
        contract_type: Some("purchase".into()),
        status: None,
        date_from: None,
        date_to: None,
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

    let (items, total) = ContractService::list_contracts(&pool, &filter, &params)
        .await
        .expect("list_contracts must succeed");
    assert_eq!(total, 1);
    assert_eq!(items[0].contract_type, "purchase");
}

#[tokio::test]
async fn list_contracts_pagination_page_size() {
    let pool = common::test_pool().await;

    common::seed_contract(&pool, "CT-LST-030", "sales", "A", "draft")
        .await
        .unwrap();
    common::seed_contract(&pool, "CT-LST-031", "sales", "B", "draft")
        .await
        .unwrap();
    common::seed_contract(&pool, "CT-LST-032", "sales", "C", "draft")
        .await
        .unwrap();

    let filter = ContractFilterParams {
        q: None,
        contract_type: None,
        status: None,
        date_from: None,
        date_to: None,
        page: Some(1),
        page_size: Some(2),
        sort_by: None,
        sort_order: None,
    };
    let params = PaginationParams {
        page: Some(1),
        page_size: Some(2),
        sort_by: None,
        sort_order: None,
    };

    let (items, total) = ContractService::list_contracts(&pool, &filter, &params)
        .await
        .expect("list_contracts must succeed");
    assert_eq!(total, 3);
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn list_contracts_empty_when_no_match() {
    let pool = common::test_pool().await;

    let filter = ContractFilterParams {
        q: None,
        contract_type: Some("nonexistent".into()),
        status: None,
        date_from: None,
        date_to: None,
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

    let (items, total) = ContractService::list_contracts(&pool, &filter, &params)
        .await
        .expect("list_contracts must succeed");
    assert_eq!(total, 0);
    assert!(items.is_empty());
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// update_status (lifecycle)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn update_status_draft_to_active() {
    let pool = common::test_pool().await;

    // Create a contract with sign_date set
    let dto = CreateContractRequest {
        contract_type: "sales".into(),
        title: "Status Lifecycle".into(),
        party_a: "A".into(),
        party_b: "B".into(),
        sign_date: Some("2025-01-01".into()),
        start_date: None,
        end_date: None,
        notes: None,
        items: vec![CreateContractItemRequest {
            pipe_type: "seamless".into(),
            grade: "L80".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 10,
            unit_price: None,
            notes: None,
        }],
    };

    let detail = ContractService::create_contract(&pool, &dto)
        .await
        .expect("create must succeed");
    let contract_id = detail.contract.id;

    // Activate
    let activated = ContractService::update_status(&pool, contract_id, "active")
        .await
        .expect("transition draft→active must succeed");
    assert_eq!(activated.status, "active");

    // Complete
    let completed = ContractService::update_status(&pool, contract_id, "completed")
        .await
        .expect("transition active→completed must succeed");
    assert_eq!(completed.status, "completed");
}

#[tokio::test]
async fn update_status_draft_to_terminated() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-STA-010", "sales", "Term Test", "draft")
        .await
        .unwrap();

    // Directly set sign_date for the termination test
    // Since we can't go through service for draft→terminated without sign_date anyway,
    // let's set sign_date first
    let update = UpdateContractRequest {
        title: None,
        party_a: None,
        party_b: None,
        sign_date: Some("2025-01-01".into()),
        start_date: None,
        end_date: None,
        notes: None,
    };
    ContractService::update_contract(&pool, contract_id, &update)
        .await
        .expect("set sign date");

    // Activate
    ContractService::update_status(&pool, contract_id, "active")
        .await
        .expect("activate must succeed");

    // Terminate
    let terminated = ContractService::update_status(&pool, contract_id, "terminated")
        .await
        .expect("transition active→terminated must succeed");
    assert_eq!(terminated.status, "terminated");
}

#[tokio::test]
async fn update_status_illegal_transition_rejected() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-STA-020", "sales", "Bad Trans", "draft")
        .await
        .unwrap();

    // Can't skip to completed directly from draft
    let err = ContractService::update_status(&pool, contract_id, "completed")
        .await
        .expect_err("transition draft→completed must reject");
    assert!(err.to_string().contains("Cannot transition"));
}

#[tokio::test]
async fn update_status_invalid_status_rejected() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-STA-030", "sales", "Bad Status", "draft")
        .await
        .unwrap();

    let err = ContractService::update_status(&pool, contract_id, "invalid_status")
        .await
        .expect_err("invalid status must reject");
    assert!(err.to_string().contains("Invalid status"));
}

#[tokio::test]
async fn update_status_activation_requires_sign_date() {
    let pool = common::test_pool().await;

    // Create a contract without sign_date
    let dto = CreateContractRequest {
        contract_type: "sales".into(),
        title: "No Sign Date".into(),
        party_a: "A".into(),
        party_b: "B".into(),
        sign_date: None,
        start_date: None,
        end_date: None,
        notes: None,
        items: vec![CreateContractItemRequest {
            pipe_type: "seamless".into(),
            grade: "L80".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 5,
            unit_price: None,
            notes: None,
        }],
    };

    let detail = ContractService::create_contract(&pool, &dto)
        .await
        .expect("create must succeed");

    let err = ContractService::update_status(&pool, detail.contract.id, "active")
        .await
        .expect_err("activation without sign date must reject");
    assert!(err.to_string().contains("sign date"));
}

#[tokio::test]
async fn update_status_nonexistent_fails() {
    let pool = common::test_pool().await;

    let err = ContractService::update_status(&pool, 99999, "active")
        .await
        .expect_err("must fail for non-existent contract");
    assert!(err.to_string().contains("not found"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// add_item / update_item / delete_item
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn add_item_to_draft_contract() {
    let pool = common::test_pool().await;

    let dto = CreateContractRequest {
        contract_type: "sales".into(),
        title: "Add Item Test".into(),
        party_a: "A".into(),
        party_b: "B".into(),
        sign_date: None,
        start_date: None,
        end_date: None,
        notes: None,
        items: vec![CreateContractItemRequest {
            pipe_type: "seamless".into(),
            grade: "L80".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 10,
            unit_price: Some(500.0),
            notes: None,
        }],
    };

    let detail = ContractService::create_contract(&pool, &dto)
        .await
        .expect("create must succeed");
    let contract_id = detail.contract.id;

    let new_item = CreateContractItemRequest {
        pipe_type: "screen".into(),
        grade: "N80".into(),
        od: 177.8,
        wt: 9.19,
        quantity: 5,
        unit_price: Some(800.0),
        notes: Some("added later".into()),
    };

    let item = ContractService::add_item(&pool, contract_id, &new_item)
        .await
        .expect("add_item must succeed");

    assert!(item.id > 0);
    assert_eq!(item.contract_id, contract_id);
    assert_eq!(item.pipe_type, "screen");
    assert_eq!(item.grade, "N80");
    assert_eq!(item.quantity, 5);
    assert_eq!(item.unit_price, Some(800.0));
}

#[tokio::test]
async fn add_item_zero_quantity_rejected() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-ITM-001", "sales", "Item Test", "draft")
        .await
        .unwrap();

    let new_item = CreateContractItemRequest {
        pipe_type: "seamless".into(),
        grade: "L80".into(),
        od: 177.8,
        wt: 9.19,
        quantity: 0,
        unit_price: None,
        notes: None,
    };

    let err = ContractService::add_item(&pool, contract_id, &new_item)
        .await
        .expect_err("zero quantity must reject");
    assert!(err.to_string().contains("Quantity must be positive"));
}

#[tokio::test]
async fn add_item_to_active_contract_rejected() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-ITM-002", "sales", "Active Item", "active")
        .await
        .unwrap();

    let new_item = CreateContractItemRequest {
        pipe_type: "seamless".into(),
        grade: "L80".into(),
        od: 177.8,
        wt: 9.19,
        quantity: 5,
        unit_price: None,
        notes: None,
    };

    let err = ContractService::add_item(&pool, contract_id, &new_item)
        .await
        .expect_err("must reject add to active contract");
    assert!(err.to_string().contains("Cannot add items"));
}

#[tokio::test]
async fn update_item_in_draft_contract() {
    let pool = common::test_pool().await;

    let dto = CreateContractRequest {
        contract_type: "purchase".into(),
        title: "Update Item".into(),
        party_a: "A".into(),
        party_b: "B".into(),
        sign_date: None,
        start_date: None,
        end_date: None,
        notes: None,
        items: vec![CreateContractItemRequest {
            pipe_type: "seamless".into(),
            grade: "L80".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 10,
            unit_price: Some(500.0),
            notes: None,
        }],
    };

    let detail = ContractService::create_contract(&pool, &dto)
        .await
        .expect("create must succeed");
    let contract_id = detail.contract.id;
    let item_id = detail.items[0].id;

    let update = UpdateContractItemRequest {
        pipe_type: None,
        grade: Some("N80".into()),
        od: None,
        wt: None,
        quantity: Some(20),
        unit_price: Some(450.0),
        notes: Some("updated".into()),
    };

    let updated = ContractService::update_item(&pool, contract_id, item_id, &update)
        .await
        .expect("update_item must succeed");

    assert_eq!(updated.grade, "N80");
    assert_eq!(updated.quantity, 20);
    assert_eq!(updated.unit_price, Some(450.0));
    assert_eq!(updated.notes.as_deref(), Some("updated"));
}

#[tokio::test]
async fn update_item_nonexistent_fails() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-ITM-010", "sales", "NotFound", "draft")
        .await
        .unwrap();

    let update = UpdateContractItemRequest {
        pipe_type: None,
        grade: None,
        od: None,
        wt: None,
        quantity: None,
        unit_price: None,
        notes: None,
    };

    let err = ContractService::update_item(&pool, contract_id, 99999, &update)
        .await
        .expect_err("must fail for non-existent item");
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn update_item_wrong_contract_rejected() {
    let pool = common::test_pool().await;

    let contract_a = common::seed_contract(&pool, "CT-ITM-020", "sales", "A", "draft")
        .await
        .unwrap();
    let contract_b = common::seed_contract(&pool, "CT-ITM-021", "sales", "B", "draft")
        .await
        .unwrap();

    let item_id = common::seed_contract_item(&pool, contract_a, "seamless", "L80", 5, Some(100.0))
        .await
        .unwrap();

    let update = UpdateContractItemRequest {
        pipe_type: None,
        grade: None,
        od: None,
        wt: None,
        quantity: None,
        unit_price: None,
        notes: None,
    };

    let err = ContractService::update_item(&pool, contract_b, item_id, &update)
        .await
        .expect_err("must reject item from wrong contract");
    assert!(err.to_string().contains("does not belong"));
}

#[tokio::test]
async fn delete_item_from_draft_contract() {
    let pool = common::test_pool().await;

    let dto = CreateContractRequest {
        contract_type: "sales".into(),
        title: "Delete Item".into(),
        party_a: "A".into(),
        party_b: "B".into(),
        sign_date: None,
        start_date: None,
        end_date: None,
        notes: None,
        items: vec![
            CreateContractItemRequest {
                pipe_type: "seamless".into(),
                grade: "L80".into(),
                od: 177.8,
                wt: 9.19,
                quantity: 10,
                unit_price: Some(100.0),
                notes: None,
            },
            CreateContractItemRequest {
                pipe_type: "screen".into(),
                grade: "N80".into(),
                od: 177.8,
                wt: 9.19,
                quantity: 5,
                unit_price: Some(200.0),
                notes: None,
            },
        ],
    };

    let detail = ContractService::create_contract(&pool, &dto)
        .await
        .expect("create must succeed");
    let contract_id = detail.contract.id;
    let item_id = detail.items[0].id;

    ContractService::delete_item(&pool, contract_id, item_id)
        .await
        .expect("delete_item must succeed");

    // Verify item is gone
    let fetched = ContractService::get_contract_detail(&pool, contract_id)
        .await
        .expect("get_contract_detail must succeed");
    assert_eq!(fetched.items.len(), 1);
    assert_eq!(fetched.items[0].id, detail.items[1].id);
}

#[tokio::test]
async fn delete_item_nonexistent_fails() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-ITM-030", "sales", "DelNotFound", "draft")
        .await
        .unwrap();

    let err = ContractService::delete_item(&pool, contract_id, 99999)
        .await
        .expect_err("must fail for non-existent item");
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn delete_item_wrong_contract_rejected() {
    let pool = common::test_pool().await;

    let contract_a = common::seed_contract(&pool, "CT-ITM-040", "sales", "A", "draft")
        .await
        .unwrap();
    let contract_b = common::seed_contract(&pool, "CT-ITM-041", "sales", "B", "draft")
        .await
        .unwrap();

    let item_id = common::seed_contract_item(&pool, contract_a, "seamless", "L80", 3, None)
        .await
        .unwrap();

    let err = ContractService::delete_item(&pool, contract_b, item_id)
        .await
        .expect_err("must reject delete item from wrong contract");
    assert!(err.to_string().contains("does not belong"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// add_payment / update_payment / delete_payment
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn add_payment_to_contract() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-PAY-001", "sales", "Payment Test", "draft")
        .await
        .unwrap();

    let pay_dto = CreatePaymentRequest {
        due_date: "2025-07-01".into(),
        amount: 50000.0,
        payment_type: "milestone".into(),
        notes: Some("first milestone".into()),
    };

    let payment = ContractService::add_payment(&pool, contract_id, &pay_dto)
        .await
        .expect("add_payment must succeed");

    assert!(payment.id > 0);
    assert_eq!(payment.contract_id, contract_id);
    assert_eq!(payment.due_date, "2025-07-01");
    assert_eq!(payment.amount, 50000.0);
    assert_eq!(payment.payment_type, "milestone");
    assert_eq!(payment.is_paid, 0);
    assert!(payment.paid_date.is_none());
    assert_eq!(payment.notes.as_deref(), Some("first milestone"));
}

#[tokio::test]
async fn add_payment_negative_amount_rejected() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-PAY-002", "sales", "Neg Amt", "draft")
        .await
        .unwrap();

    let pay_dto = CreatePaymentRequest {
        due_date: "2025-07-01".into(),
        amount: -100.0,
        payment_type: "deposit".into(),
        notes: None,
    };

    let err = ContractService::add_payment(&pool, contract_id, &pay_dto)
        .await
        .expect_err("negative amount must reject");
    assert!(err.to_string().contains("amount must be positive"));
}

#[tokio::test]
async fn add_payment_invalid_type_rejected() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-PAY-003", "sales", "Bad Type", "draft")
        .await
        .unwrap();

    let pay_dto = CreatePaymentRequest {
        due_date: "2025-07-01".into(),
        amount: 1000.0,
        payment_type: "invalid_type".into(),
        notes: None,
    };

    let err = ContractService::add_payment(&pool, contract_id, &pay_dto)
        .await
        .expect_err("invalid payment type must reject");
    assert!(err.to_string().contains("Invalid payment type"));
}

#[tokio::test]
async fn add_payment_to_terminated_contract_rejected() {
    let pool = common::test_pool().await;

    let contract_id =
        common::seed_contract(&pool, "CT-PAY-004", "sales", "Terminated", "terminated")
            .await
            .unwrap();

    let pay_dto = CreatePaymentRequest {
        due_date: "2025-07-01".into(),
        amount: 1000.0,
        payment_type: "deposit".into(),
        notes: None,
    };

    let err = ContractService::add_payment(&pool, contract_id, &pay_dto)
        .await
        .expect_err("must reject add to terminated contract");
    assert!(err.to_string().contains("Cannot add payments"));
}

#[tokio::test]
async fn update_payment_fields() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-PAY-010", "sales", "Upd Pay", "draft")
        .await
        .unwrap();

    let pay_dto = CreatePaymentRequest {
        due_date: "2025-08-01".into(),
        amount: 30000.0,
        payment_type: "deposit".into(),
        notes: Some("initial".into()),
    };
    let payment = ContractService::add_payment(&pool, contract_id, &pay_dto)
        .await
        .expect("add_payment must succeed");

    let update = UpdatePaymentRequest {
        due_date: Some("2025-09-01".into()),
        amount: Some(35000.0),
        payment_type: Some("milestone".into()),
        is_paid: Some(1),
        paid_date: Some("2025-09-01".into()),
        notes: Some("updated".into()),
    };

    let updated = ContractService::update_payment(&pool, contract_id, payment.id, &update)
        .await
        .expect("update_payment must succeed");

    assert_eq!(updated.id, payment.id);
    assert_eq!(updated.due_date, "2025-09-01");
    assert_eq!(updated.amount, 35000.0);
    assert_eq!(updated.payment_type, "milestone");
    assert_eq!(updated.is_paid, 1);
    assert_eq!(updated.paid_date.as_deref(), Some("2025-09-01"));
    assert_eq!(updated.notes.as_deref(), Some("updated"));
}

#[tokio::test]
async fn update_payment_wrong_contract_rejected() {
    let pool = common::test_pool().await;

    let contract_a = common::seed_contract(&pool, "CT-PAY-020", "sales", "A", "draft")
        .await
        .unwrap();
    let contract_b = common::seed_contract(&pool, "CT-PAY-021", "sales", "B", "draft")
        .await
        .unwrap();

    let pay_dto = CreatePaymentRequest {
        due_date: "2025-08-01".into(),
        amount: 1000.0,
        payment_type: "deposit".into(),
        notes: None,
    };
    let payment = ContractService::add_payment(&pool, contract_a, &pay_dto)
        .await
        .expect("add_payment must succeed");

    let update = UpdatePaymentRequest {
        due_date: None,
        amount: None,
        payment_type: None,
        is_paid: None,
        paid_date: None,
        notes: None,
    };

    let err = ContractService::update_payment(&pool, contract_b, payment.id, &update)
        .await
        .expect_err("wrong contract must reject");
    assert!(err.to_string().contains("does not belong"));
}

#[tokio::test]
async fn update_payment_nonexistent_fails() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-PAY-030", "sales", "NotFound", "draft")
        .await
        .unwrap();

    let update = UpdatePaymentRequest {
        due_date: None,
        amount: None,
        payment_type: None,
        is_paid: None,
        paid_date: None,
        notes: None,
    };

    let err = ContractService::update_payment(&pool, contract_id, 99999, &update)
        .await
        .expect_err("non-existent payment must reject");
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn update_payment_terminated_contract_rejected() {
    let pool = common::test_pool().await;

    let contract_id =
        common::seed_contract(&pool, "CT-PAY-040", "sales", "Term", "terminated")
            .await
            .unwrap();

    let err = ContractService::update_payment(&pool, contract_id, 99999, &UpdatePaymentRequest {
        due_date: None,
        amount: None,
        payment_type: None,
        is_paid: None,
        paid_date: None,
        notes: None,
    })
    .await
    .expect_err("must reject update on terminated contract");
    assert!(err.to_string().contains("Cannot modify payments"));
}

#[tokio::test]
async fn delete_payment_success() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-PAY-050", "sales", "Del Pay", "draft")
        .await
        .unwrap();

    let pay_dto = CreatePaymentRequest {
        due_date: "2025-10-01".into(),
        amount: 20000.0,
        payment_type: "final".into(),
        notes: None,
    };
    let payment = ContractService::add_payment(&pool, contract_id, &pay_dto)
        .await
        .expect("add_payment must succeed");

    ContractService::delete_payment(&pool, contract_id, payment.id)
        .await
        .expect("delete_payment must succeed");

    // Verify payment is gone via get_payments
    let payments = ContractService::get_payments(&pool, contract_id)
        .await
        .expect("get_payments must succeed");
    assert!(payments.is_empty());
}

#[tokio::test]
async fn delete_payment_nonexistent_fails() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-PAY-060", "sales", "DelNF", "draft")
        .await
        .unwrap();

    let err = ContractService::delete_payment(&pool, contract_id, 99999)
        .await
        .expect_err("must fail for non-existent payment");
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn delete_payment_terminated_contract_rejected() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-PAY-070", "sales", "TermDel", "terminated")
        .await
        .unwrap();

    let err = ContractService::delete_payment(&pool, contract_id, 99999)
        .await
        .expect_err("must reject delete on terminated contract");
    assert!(err.to_string().contains("Cannot delete payments"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// get_payments
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn get_payments_by_contract_id() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-PAY-080", "sales", "Get Pay", "draft")
        .await
        .unwrap();

    // Add two payments
    ContractService::add_payment(&pool, contract_id, &CreatePaymentRequest {
        due_date: "2025-11-01".into(),
        amount: 10000.0,
        payment_type: "deposit".into(),
        notes: None,
    })
    .await
    .expect("add first payment");

    ContractService::add_payment(&pool, contract_id, &CreatePaymentRequest {
        due_date: "2025-12-01".into(),
        amount: 20000.0,
        payment_type: "final".into(),
        notes: None,
    })
    .await
    .expect("add second payment");

    let payments = ContractService::get_payments(&pool, contract_id)
        .await
        .expect("get_payments must succeed");

    assert_eq!(payments.len(), 2);
    assert!(payments.iter().all(|p| p.contract_id == contract_id));
}

#[tokio::test]
async fn get_payments_empty_when_none() {
    let pool = common::test_pool().await;

    let contract_id = common::seed_contract(&pool, "CT-PAY-090", "sales", "Empty Pay", "draft")
        .await
        .unwrap();

    let payments = ContractService::get_payments(&pool, contract_id)
        .await
        .expect("get_payments must succeed");
    assert!(payments.is_empty());
}

#[tokio::test]
async fn get_payments_nonexistent_contract_fails() {
    let pool = common::test_pool().await;

    let err = ContractService::get_payments(&pool, 99999)
        .await
        .expect_err("must fail for non-existent contract");
    assert!(err.to_string().contains("not found"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Full lifecycle: create → add items → add payments → list → update status → soft delete
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn contract_lifecycle() {
    let pool = common::test_pool().await;

    // ── Create with items ──
    let dto = CreateContractRequest {
        contract_type: "sales".into(),
        title: "Lifecycle Contract".into(),
        party_a: "Seller Ltd".into(),
        party_b: "Buyer Ltd".into(),
        sign_date: Some("2025-01-01".into()),
        start_date: Some("2025-02-01".into()),
        end_date: Some("2025-12-31".into()),
        notes: Some("lifecycle test".into()),
        items: vec![
            CreateContractItemRequest {
                pipe_type: "seamless".into(),
                grade: "L80".into(),
                od: 177.8,
                wt: 9.19,
                quantity: 50,
                unit_price: Some(500.0),
                notes: None,
            },
        ],
    };

    let detail = ContractService::create_contract(&pool, &dto)
        .await
        .expect("create must succeed");
    let contract_id = detail.contract.id;
    assert_eq!(detail.items.len(), 1);

    // ── Add another item ──
    let new_item = CreateContractItemRequest {
        pipe_type: "screen".into(),
        grade: "N80".into(),
        od: 177.8,
        wt: 9.19,
        quantity: 10,
        unit_price: Some(700.0),
        notes: None,
    };
    let added_item = ContractService::add_item(&pool, contract_id, &new_item)
        .await
        .expect("add_item must succeed");
    assert!(added_item.id > 0);

    // ── Add payment ──
    let pay = ContractService::add_payment(&pool, contract_id, &CreatePaymentRequest {
        due_date: "2025-03-01".into(),
        amount: 15000.0,
        payment_type: "deposit".into(),
        notes: None,
    })
    .await
    .expect("add_payment must succeed");

    // ── Get detail ──
    let fetched = ContractService::get_contract_detail(&pool, contract_id)
        .await
        .expect("get_contract_detail must succeed");
    assert_eq!(fetched.items.len(), 2);
    assert_eq!(fetched.payments.len(), 1);

    // ── List contracts ──
    let filter = ContractFilterParams {
        q: None,
        contract_type: None,
        status: Some("draft".into()),
        date_from: None,
        date_to: None,
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
    let (items, total) = ContractService::list_contracts(&pool, &filter, &params)
        .await
        .expect("list_contracts must succeed");
    assert!(total >= 1);
    assert!(items.iter().any(|c| c.id == contract_id));

    // ── Update status: draft → active ──
    let activated = ContractService::update_status(&pool, contract_id, "active")
        .await
        .expect("draft→active must succeed");
    assert_eq!(activated.status, "active");

    // ── Update status: active → completed ──
    let completed = ContractService::update_status(&pool, contract_id, "completed")
        .await
        .expect("active→completed must succeed");
    assert_eq!(completed.status, "completed");

    // ── Verify detail still works after completion ──
    let final_detail = ContractService::get_contract_detail(&pool, contract_id)
        .await
        .expect("get_contract_detail must succeed after completion");
    assert_eq!(final_detail.contract.status, "completed");
    assert_eq!(final_detail.items.len(), 2);
    assert_eq!(final_detail.payments.len(), 1);
}
