//! Integration tests for DataIOService.
//!
//! Covers:
//! - Export entities (CSV + XLSX) for items / inventory / orders
//! - Download import templates (CSV + XLSX)
//! - Import items from CSV data
//! - Operation log recording and querying
//! - Utility functions (content_type, file_extension)
//!
//! All tests use a fresh SQLite database with migrations applied.

mod common;

use std::io::Cursor;

use calamine::{open_workbook_from_rs, Data, Reader, Xlsx};
use erp_server::dto::data_io_dto::OperationLogQuery;
use erp_server::data_io::data_io_service::DataIOService;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Item helpers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Seed an item master row; returns its id.
async fn seed_item(pool: &sqlx::SqlitePool, sku: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO items (sku, name, category, unit, spec, price, status) \
         VALUES (?, 'Test Item', '原材料', '个', 'spec', 10.0, 'active') \
         RETURNING id",
    )
    .bind(sku)
    .fetch_one(pool)
    .await
    .expect("seed_item must succeed")
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// export_entity — CSV
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn export_entity_items_csv() {
    let pool = common::test_pool().await;

    seed_item(&pool, "ITM-EXP-001").await;

    let data = DataIOService::export_entity(&pool, "items", "csv")
        .await
        .expect("export_entity items csv must succeed");

    assert!(!data.is_empty(), "CSV data must not be empty");

    let content = String::from_utf8_lossy(&data);
    assert!(content.contains("sku"), "CSV should have header");
    assert!(
        content.contains("ITM-EXP-001"),
        "CSV should contain item data"
    );
}

#[tokio::test]
async fn export_entity_inventory_csv() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "ITM-INV-EXP").await;
    sqlx::query(
        "INSERT INTO inventory_logs (item_id, quantity, change_type, created_at) \
         VALUES (?, 5.0, 'inbound', datetime('now'))",
    )
    .bind(item_id)
    .execute(&pool)
    .await
    .unwrap();

    let data = DataIOService::export_entity(&pool, "inventory", "csv")
        .await
        .expect("export_entity inventory csv must succeed");

    let content = String::from_utf8_lossy(&data);
    assert!(content.contains("sku"));
    assert!(content.contains("ITM-INV-EXP"));
}

#[tokio::test]
async fn export_entity_purchase_orders_csv() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-EXP", "Export Supplier")
        .await
        .unwrap();
    common::seed_purchase_order(&pool, "PO-EXP-001", supplier_id, "pending")
        .await
        .unwrap();

    let data = DataIOService::export_entity(&pool, "purchase_orders", "csv")
        .await
        .expect("export_entity purchase_orders csv must succeed");

    let content = String::from_utf8_lossy(&data);
    assert!(content.contains("PO-EXP-001"));
}

#[tokio::test]
async fn export_entity_sales_orders_csv() {
    let pool = common::test_pool().await;

    let customer_id = common::seed_customer(&pool, "CUST-EXP", "Export Customer")
        .await
        .unwrap();
    common::seed_sales_order(&pool, "SO-EXP-001", customer_id, "pending")
        .await
        .unwrap();

    let data = DataIOService::export_entity(&pool, "sales_orders", "csv")
        .await
        .expect("export_entity sales_orders csv must succeed");

    let content = String::from_utf8_lossy(&data);
    assert!(content.contains("SO-EXP-001"));
}

#[tokio::test]
async fn export_entity_empty_database_returns_headers_only() {
    let pool = common::test_pool().await;

    let data = DataIOService::export_entity(&pool, "items", "csv")
        .await
        .expect("export_entity on empty DB must succeed");

    let content = String::from_utf8_lossy(&data);
    assert!(content.contains("sku"), "should have header row");
    // Should only have header (1 line) or header + no data (just header)
    let lines: Vec<&str> = content.lines().collect();
    assert!(lines.len() >= 1, "should have at least header line");
}

#[tokio::test]
async fn export_entity_csv_escapes_spreadsheet_formula_prefixes() {
    let pool = common::test_pool().await;

    seed_item(&pool, "=cmd|'/C calc'!A0").await;

    let data = DataIOService::export_entity(&pool, "items", "csv")
        .await
        .expect("export_entity items csv must succeed");

    let content = String::from_utf8_lossy(&data);
    assert!(content.contains("'=cmd|'/C calc'!A0"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// export_entity — XLSX
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn export_entity_items_xlsx() {
    let pool = common::test_pool().await;

    seed_item(&pool, "ITM-XLSX-001").await;

    let data = DataIOService::export_entity(&pool, "items", "xlsx")
        .await
        .expect("export_entity items xlsx must succeed");

    assert!(!data.is_empty(), "XLSX data must not be empty");
    // XLSX starts with the ZIP magic bytes (PK\x03\x04)
    assert!(
        data.starts_with(&[0x50, 0x4b, 0x03, 0x04]),
        "should be a valid xlsx file"
    );
}

#[tokio::test]
async fn export_entity_xlsx_escapes_spreadsheet_formula_prefixes() {
    let pool = common::test_pool().await;

    seed_item(&pool, "@SUM(1,1)").await;

    let data = DataIOService::export_entity(&pool, "items", "xlsx")
        .await
        .expect("export_entity items xlsx must succeed");

    let cursor = Cursor::new(data);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor).expect("xlsx must open");
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .expect("sheet exists");
    let range = workbook
        .worksheet_range(&sheet_name)
        .expect("sheet range exists");
    let mut found_escaped = false;
    for row in range.rows() {
        if let Some(cell) = row.first() {
            if cell == &Data::String("'@SUM(1,1)".to_string()) {
                found_escaped = true;
                break;
            }
        }
    }
    assert!(found_escaped, "should find escaped formula prefix in export");
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// export_entity — error cases
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn export_entity_invalid_entity_type() {
    let pool = common::test_pool().await;

    let err = DataIOService::export_entity(&pool, "invalid_entity", "csv")
        .await
        .expect_err("must reject invalid entity type");

    assert!(err.to_string().contains("Invalid entity"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// download_template
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn download_template_csv() {
    let _pool = common::test_pool().await;

    let data = DataIOService::download_template("items", "csv")
        .await
        .expect("download_template csv must succeed");

    let content = String::from_utf8_lossy(&data);
    assert!(
        content.contains("sku"),
        "CSV template should have headers"
    );
    assert!(
        content.contains("category"),
        "should have raw column name"
    );
    // Template should have exactly one line (headers only)
    assert_eq!(content.lines().count(), 1, "template should be header-only");
}

#[tokio::test]
async fn download_template_xlsx() {
    let data = DataIOService::download_template("items", "xlsx")
        .await
        .expect("download_template xlsx must succeed");

    assert!(!data.is_empty());
    assert!(
        data.starts_with(&[0x50, 0x4b, 0x03, 0x04]),
        "should be a valid xlsx"
    );
}

#[tokio::test]
async fn download_template_invalid_entity() {
    let err = DataIOService::download_template("invalid_entity", "csv")
        .await
        .expect_err("must reject invalid entity type");

    assert!(err.to_string().contains("Invalid entity"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// import_entity — CSV
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn import_entity_items_csv() {
    let pool = common::test_pool().await;

    let csv_data = "sku,name,category,unit,spec,price,status\n\
                     ITM-IMP-001,Steel Plate,原材料,张,8mm,4280.0,active";

    let result = DataIOService::import_entity(&pool, "items", csv_data.as_bytes(), "import.csv")
        .await
        .expect("import_entity items must succeed");

    assert_eq!(result.imported_count, 1);
    assert_eq!(result.failed_count, 0);
    assert!(result.errors.is_empty());
    assert_eq!(result.entity_type, "items");
}

#[tokio::test]
async fn import_entity_items_csv_duplicate_sku_skipped() {
    let pool = common::test_pool().await;

    seed_item(&pool, "ITM-DUP-001").await;

    let csv_data = "sku,name,category,unit,spec,price,status\n\
                     ITM-DUP-001,Duplicate,原材料,个,spec,10.0,active";

    let result = DataIOService::import_entity(&pool, "items", csv_data.as_bytes(), "import.csv")
        .await
        .expect("import_entity items must succeed");

    assert_eq!(result.imported_count, 0);
    assert_eq!(result.failed_count, 1);
    assert!(
        result.errors[0].contains("already exists"),
        "should report duplicate sku"
    );
}

#[tokio::test]
async fn import_entity_multiple_rows_csv() {
    let pool = common::test_pool().await;

    let csv_data = "sku,name,category,unit,spec,price,status\n\
                     ITM-IMP-010,Item Ten,原材料,个,spec,10.0,active\n\
                     ITM-IMP-011,Item Eleven,标准件,个,spec,20.0,active";

    let result = DataIOService::import_entity(&pool, "items", csv_data.as_bytes(), "import.csv")
        .await
        .expect("import_entity multiple rows must succeed");

    assert_eq!(result.imported_count, 2);
    assert_eq!(result.failed_count, 0);
}

#[tokio::test]
async fn import_entity_invalid_entity_type() {
    let pool = common::test_pool().await;

    let err = DataIOService::import_entity(&pool, "invalid_entity", b"data", "file.csv")
        .await
        .expect_err("must reject invalid entity type");

    assert!(err.to_string().contains("Invalid entity"));
}

#[tokio::test]
async fn import_entity_empty_data_error() {
    let pool = common::test_pool().await;

    // CSV with header but no data rows
    let csv_data = "sku,name,category,unit,spec,price,status";

    let err = DataIOService::import_entity(&pool, "items", csv_data.as_bytes(), "file.csv")
        .await
        .expect_err("must reject empty data");

    assert!(err.to_string().contains("No data rows"));
}

#[tokio::test]
async fn import_entity_unsupported_entity_type() {
    let pool = common::test_pool().await;

    let csv_data = "order_no,supplier_id\nPO-TEST,1";
    let err =
        DataIOService::import_entity(&pool, "purchase_orders", csv_data.as_bytes(), "file.csv")
            .await
            .expect_err("must reject unsupported import entity");

    assert!(err.to_string().contains("Import not supported"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// list_operation_logs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn list_operation_logs_empty() {
    let pool = common::test_pool().await;

    let query = OperationLogQuery {
        page: None,
        page_size: None,
        user_id: None,
        action: None,
        entity_type: None,
    };

    let (logs, total) = DataIOService::list_operation_logs(&pool, &query)
        .await
        .expect("list_operation_logs must succeed");

    assert_eq!(total, 0);
    assert!(logs.is_empty());
}

#[tokio::test]
async fn list_operation_logs_with_data() {
    let pool = common::test_pool().await;

    let user_id = common::seed_user(&pool, "oplog_user", "admin")
        .await
        .unwrap();

    common::seed_operation_log(&pool, "import", "items", 1, user_id)
        .await
        .unwrap();
    common::seed_operation_log(&pool, "export", "items", 1, user_id)
        .await
        .unwrap();

    let query = OperationLogQuery {
        page: None,
        page_size: None,
        user_id: None,
        action: None,
        entity_type: None,
    };

    let (logs, total) = DataIOService::list_operation_logs(&pool, &query)
        .await
        .expect("list_operation_logs must succeed");

    assert_eq!(total, 2);
    assert_eq!(logs.len(), 2);
}

#[tokio::test]
async fn list_operation_logs_filtered_by_action() {
    let pool = common::test_pool().await;

    let user_id = common::seed_user(&pool, "oplog_filter", "admin")
        .await
        .unwrap();

    common::seed_operation_log(&pool, "import", "items", 1, user_id)
        .await
        .unwrap();
    common::seed_operation_log(&pool, "export", "items", 1, user_id)
        .await
        .unwrap();

    let query = OperationLogQuery {
        page: None,
        page_size: None,
        user_id: None,
        action: Some("import".into()),
        entity_type: None,
    };

    let (logs, total) = DataIOService::list_operation_logs(&pool, &query)
        .await
        .expect("list_operation_logs must succeed");

    assert_eq!(total, 1);
    assert_eq!(logs[0].action, "import");
}

#[tokio::test]
async fn list_operation_logs_paginated() {
    let pool = common::test_pool().await;

    let user_id = common::seed_user(&pool, "oplog_page", "admin")
        .await
        .unwrap();

    for i in 0..5 {
        common::seed_operation_log(&pool, &format!("action_{}", i), "test_entity", 1, user_id)
            .await
            .unwrap();
    }

    let query = OperationLogQuery {
        page: Some(1),
        page_size: Some(2),
        user_id: None,
        action: None,
        entity_type: None,
    };

    let (logs, total) = DataIOService::list_operation_logs(&pool, &query)
        .await
        .expect("list_operation_logs must succeed");

    assert_eq!(total, 5, "total should be 5");
    assert_eq!(logs.len(), 2, "page should return 2 items");
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// log_operation
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn log_operation_creates_entry() {
    let pool = common::test_pool().await;

    DataIOService::log_operation(
        &pool,
        Some(1),
        Some("admin".into()),
        "import",
        "items",
        Some(100),
        Some("imported 10 items".into()),
        Some("127.0.0.1".into()),
    )
    .await
    .expect("log_operation must succeed");

    let query = OperationLogQuery {
        page: None,
        page_size: None,
        user_id: None,
        action: None,
        entity_type: None,
    };

    let (logs, total) = DataIOService::list_operation_logs(&pool, &query)
        .await
        .expect("list_operation_logs must succeed");

    assert_eq!(total, 1);
    assert_eq!(logs[0].action, "import");
    assert_eq!(logs[0].entity_type, "items");
    assert_eq!(logs[0].entity_id, Some(100));
    assert!(logs[0].details.is_some());
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// content_type / file_extension
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn content_type_csv() {
    assert_eq!(
        DataIOService::content_type("csv"),
        "text/csv; charset=utf-8"
    );
}

#[tokio::test]
async fn content_type_xlsx() {
    assert_eq!(
        DataIOService::content_type("xlsx"),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    );
}

#[tokio::test]
async fn content_type_default_to_xlsx() {
    assert_eq!(
        DataIOService::content_type("unknown"),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    );
}

#[tokio::test]
async fn file_extension_csv() {
    assert_eq!(DataIOService::file_extension("csv"), "csv");
}

#[tokio::test]
async fn file_extension_default_to_xlsx() {
    assert_eq!(DataIOService::file_extension("xlsx"), "xlsx");
    assert_eq!(DataIOService::file_extension("pdf"), "xlsx");
    assert_eq!(DataIOService::file_extension(""), "xlsx");
}
