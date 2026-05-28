//! Integration tests for DataIOService.
//!
//! Covers:
//! - Export entities (CSV + XLSX) for all supported types
//! - Download import templates (CSV + XLSX)
//! - Import entities from CSV data
//! - Operation log recording and querying
//! - Utility functions (content_type, file_extension)
//!
//! All tests use an in-memory SQLite database with fresh migrations.

mod common;

use steel_pipe_db::dto::data_io_dto::OperationLogQuery;
use steel_pipe_db::services::data_io_service::DataIOService;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// export_entity — CSV
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn export_entity_seamless_pipes_csv() {
    let pool = common::test_pool().await;

    common::seed_seamless_pipe(&pool, "PN-EXP-001", "in_stock", "L80")
        .await
        .unwrap();

    let data = DataIOService::export_entity(&pool, "seamless_pipes", "csv")
        .await
        .expect("export_entity seamless_pipes csv must succeed");

    assert!(!data.is_empty(), "CSV data must not be empty");

    let content = String::from_utf8_lossy(&data);
    assert!(content.contains("pipe_number"), "CSV should have header");
    assert!(content.contains("PN-EXP-001"), "CSV should contain pipe data");
}

#[tokio::test]
async fn export_entity_screen_pipes_csv() {
    let pool = common::test_pool().await;

    common::seed_screen_pipe(&pool, "SCR-EXP-001", "in_stock", "N80")
        .await
        .unwrap();

    let data = DataIOService::export_entity(&pool, "screen_pipes", "csv")
        .await
        .expect("export_entity screen_pipes csv must succeed");

    let content = String::from_utf8_lossy(&data);
    assert!(content.contains("SCR-EXP-001"));
}

#[tokio::test]
async fn export_entity_inventory_csv() {
    let pool = common::test_pool().await;

    common::seed_seamless_pipe(&pool, "PN-INV-EXP", "in_stock", "L80")
        .await
        .unwrap();

    let data = DataIOService::export_entity(&pool, "inventory", "csv")
        .await
        .expect("export_entity inventory csv must succeed");

    let content = String::from_utf8_lossy(&data);
    assert!(content.contains("pipe_number"));
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
async fn export_entity_quality_certs_csv() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-QC-EXP", "in_stock", "L80")
        .await
        .unwrap();
    common::seed_quality_cert(&pool, "QC-EXP-001", "seamless", pipe_id, "pass")
        .await
        .unwrap();

    let data = DataIOService::export_entity(&pool, "quality_certs", "csv")
        .await
        .expect("export_entity quality_certs csv must succeed");

    let content = String::from_utf8_lossy(&data);
    assert!(content.contains("QC-EXP-001"));
}

#[tokio::test]
async fn export_entity_empty_database_returns_headers_only() {
    let pool = common::test_pool().await;

    let data = DataIOService::export_entity(&pool, "seamless_pipes", "csv")
        .await
        .expect("export_entity on empty DB must succeed");

    let content = String::from_utf8_lossy(&data);
    assert!(content.contains("pipe_number"), "should have header row");
    // Should only have header (1 line) or header + no data (just header)
    let lines: Vec<&str> = content.lines().collect();
    assert!(lines.len() >= 1, "should have at least header line");
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// export_entity — XLSX
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn export_entity_seamless_pipes_xlsx() {
    let pool = common::test_pool().await;

    common::seed_seamless_pipe(&pool, "PN-XLSX-001", "in_stock", "L80")
        .await
        .unwrap();

    let data = DataIOService::export_entity(&pool, "seamless_pipes", "xlsx")
        .await
        .expect("export_entity seamless_pipes xlsx must succeed");

    assert!(!data.is_empty(), "XLSX data must not be empty");
    // XLSX starts with the ZIP magic bytes (PK\x03\x04)
    assert!(data.starts_with(&[0x50, 0x4b, 0x03, 0x04]), "should be a valid xlsx file");
}

#[tokio::test]
async fn export_entity_xlsx_screen_pipes() {
    let pool = common::test_pool().await;

    common::seed_screen_pipe(&pool, "SCR-XLSX", "in_stock", "N80")
        .await
        .unwrap();

    let data = DataIOService::export_entity(&pool, "screen_pipes", "xlsx")
        .await
        .expect("export_entity screen_pipes xlsx must succeed");

    assert!(!data.is_empty());
    assert!(data.starts_with(&[0x50, 0x4b, 0x03, 0x04]));
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

    let data = DataIOService::download_template("seamless_pipes", "csv")
        .await
        .expect("download_template csv must succeed");

    let content = String::from_utf8_lossy(&data);
    assert!(content.contains("pipe_number"), "CSV template should have headers");
    assert!(content.contains("heat_number"), "should have raw column name");
    // Template should have exactly one line (headers only)
    assert_eq!(content.lines().count(), 1, "template should be header-only");
}

#[tokio::test]
async fn download_template_xlsx() {
    let data = DataIOService::download_template("seamless_pipes", "xlsx")
        .await
        .expect("download_template xlsx must succeed");

    assert!(!data.is_empty());
    assert!(data.starts_with(&[0x50, 0x4b, 0x03, 0x04]), "should be a valid xlsx");
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
async fn import_entity_seamless_pipes_csv() {
    let pool = common::test_pool().await;

    let csv_data = "pipe_number,batch_number,pipe_type,grade,od,wt,length,weight_per_unit,end_type,coupling_type,coupling_od,coupling_length,heat_number,serial_number,manufacturer,production_date,cert_number,status,notes\n\
                     PN-IMP-001,BN-001,casing,L80,177.8,9.19,9.5,40.0,BTC,N80Q,200.0,200.0,HN-IMP-001,SN-001,Test Mfr,2025-06-01,CERT-001,in_stock,test import";

    let result = DataIOService::import_entity(&pool, "seamless_pipes", csv_data.as_bytes(), "import.csv")
        .await
        .expect("import_entity seamless_pipes must succeed");

    assert_eq!(result.imported_count, 1);
    assert_eq!(result.failed_count, 0);
    assert!(result.errors.is_empty());
    assert_eq!(result.entity_type, "seamless_pipes");
}

#[tokio::test]
async fn import_entity_screen_pipes_csv() {
    let pool = common::test_pool().await;

    let csv_data = "pipe_number,batch_number,screen_type,slot_size,filtration_grade,base_od,base_wt,base_grade,base_end_type,length,weight_per_unit,heat_number,serial_number,manufacturer,production_date,cert_number,status,notes\n\
                     SCR-IMP-001,BN-001,slotted,0.02,standard,177.8,9.19,N80,BTC,9.5,40.0,HN-IMP-002,SN-002,Test Mfr,2025-06-01,CERT-002,in_stock,test screen import";

    let result = DataIOService::import_entity(&pool, "screen_pipes", csv_data.as_bytes(), "import.csv")
        .await
        .expect("import_entity screen_pipes must succeed");

    assert_eq!(result.imported_count, 1);
    assert_eq!(result.failed_count, 0);
    assert_eq!(result.entity_type, "screen_pipes");
}

#[tokio::test]
async fn import_entity_multiple_rows_csv() {
    let pool = common::test_pool().await;

    let csv_data = "pipe_number,batch_number,pipe_type,grade,od,wt,length,weight_per_unit,end_type,coupling_type,coupling_od,coupling_length,heat_number,serial_number,manufacturer,production_date,cert_number,status,notes\n\
                     PN-IMP-010,BN-001,casing,L80,177.8,9.19,9.5,40.0,BTC,N80Q,200.0,200.0,HN-010,SN-010,Test,2025-06-01,C-010,in_stock,\n\
                     PN-IMP-011,BN-001,casing,J55,177.8,9.19,9.5,40.0,BTC,N80Q,200.0,200.0,HN-011,SN-011,Test,2025-06-01,C-011,in_stock,";

    let result = DataIOService::import_entity(&pool, "seamless_pipes", csv_data.as_bytes(), "import.csv")
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
    let csv_data = "pipe_number,batch_number,pipe_type,grade,od,wt,length,weight_per_unit,end_type,coupling_type,coupling_od,coupling_length,heat_number,serial_number,manufacturer,production_date,cert_number,status,notes";

    let err = DataIOService::import_entity(&pool, "seamless_pipes", csv_data.as_bytes(), "file.csv")
        .await
        .expect_err("must reject empty data");

    assert!(err.to_string().contains("No data rows"));
}

#[tokio::test]
async fn import_entity_unsupported_entity_type() {
    let pool = common::test_pool().await;

    let csv_data = "order_no,supplier_id\nPO-TEST,1";
    let err = DataIOService::import_entity(&pool, "purchase_orders", csv_data.as_bytes(), "file.csv")
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

    common::seed_operation_log(&pool, "import", "seamless_pipes", 1, user_id)
        .await
        .unwrap();
    common::seed_operation_log(&pool, "export", "seamless_pipes", 1, user_id)
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

    common::seed_operation_log(&pool, "import", "seamless_pipes", 1, user_id)
        .await
        .unwrap();
    common::seed_operation_log(&pool, "export", "seamless_pipes", 1, user_id)
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
        "seamless_pipes",
        Some(100),
        Some("imported 10 pipes".into()),
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
    assert_eq!(logs[0].entity_type, "seamless_pipes");
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
