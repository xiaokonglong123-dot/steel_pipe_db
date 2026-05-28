//! Integration tests for LabelService.
//!
//! Covers:
//! - Single pipe label generation (seamless + screen)
//! - Batch label generation
//! - Quality tag generation
//! - Shipping label generation
//! - Error cases: pipe not found, invalid pipe type
//!
//! All tests use an in-memory SQLite database with fresh migrations.

mod common;

use steel_pipe_db::dto::label_dto::{BatchLabelRequest, PipeIdentifier, ShippingLabelRequest};
use steel_pipe_db::services::label_service::LabelService;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// generate_pipe_label — seamless
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn generate_pipe_label_seamless_returns_html() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-LBL-001", "in_stock", "L80")
        .await
        .unwrap();

    let html = LabelService::generate_pipe_label(&pool, "seamless", pipe_id)
        .await
        .expect("generate_pipe_label must succeed");

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("PN-LBL-001"));
    assert!(html.contains("L80"));
    assert!(html.contains("label-page"));
}

#[tokio::test]
async fn generate_pipe_label_seamless_contains_specs() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe_full(&pool, "PN-LBL-002", "in_stock", "J55", 177.8, 9.19, 9.5)
        .await
        .unwrap();

    let html = LabelService::generate_pipe_label(&pool, "seamless", pipe_id)
        .await
        .expect("generate_pipe_label must succeed");

    assert!(html.contains("177.8"));
    assert!(html.contains("9.19"));
    assert!(html.contains("9.5"));
    assert!(html.contains("J55"));
}

#[tokio::test]
async fn generate_pipe_label_screen_returns_html() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_screen_pipe(&pool, "PN-LBL-SCR-001", "in_stock", "L80")
        .await
        .unwrap();

    let html = LabelService::generate_pipe_label(&pool, "screen", pipe_id)
        .await
        .expect("generate_pipe_label must succeed");

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("PN-LBL-SCR-001"));
    assert!(html.contains("label-page"));
}

#[tokio::test]
async fn generate_pipe_label_screen_contains_specs() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_screen_pipe(&pool, "PN-LBL-SCR-002", "in_stock", "N80")
        .await
        .unwrap();

    let html = LabelService::generate_pipe_label(&pool, "screen", pipe_id)
        .await
        .expect("generate_pipe_label must succeed");

    assert!(html.contains("N80"));
    assert!(html.contains("slotted"));
}

#[tokio::test]
async fn generate_pipe_label_invalid_pipe_type() {
    let pool = common::test_pool().await;

    let err = LabelService::generate_pipe_label(&pool, "invalid_type", 1)
        .await
        .expect_err("must reject invalid pipe type");

    assert!(err.to_string().contains("Invalid pipe_type"));
}

#[tokio::test]
async fn generate_pipe_label_pipe_not_found() {
    let pool = common::test_pool().await;

    let err = LabelService::generate_pipe_label(&pool, "seamless", 99999)
        .await
        .expect_err("must reject non-existent pipe");

    assert!(err.to_string().contains("not found"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// generate_batch_labels
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn generate_batch_labels_multiple_pipes() {
    let pool = common::test_pool().await;

    let pid1 = common::seed_seamless_pipe(&pool, "PN-BAT-001", "in_stock", "L80")
        .await
        .unwrap();
    let pid2 = common::seed_screen_pipe(&pool, "PN-BAT-002", "in_stock", "J55")
        .await
        .unwrap();

    let req = BatchLabelRequest {
        pipe_ids: vec![
            PipeIdentifier { pipe_type: "seamless".into(), pipe_id: pid1 },
            PipeIdentifier { pipe_type: "screen".into(), pipe_id: pid2 },
        ],
    };

    let html = LabelService::generate_batch_labels(&pool, &req)
        .await
        .expect("generate_batch_labels must succeed");

    assert!(html.contains("PN-BAT-001"));
    assert!(html.contains("PN-BAT-002"));
    assert!(html.contains("L80"));
    assert!(html.contains("J55"));
}

#[tokio::test]
async fn generate_batch_labels_single_pipe() {
    let pool = common::test_pool().await;

    let pid = common::seed_seamless_pipe(&pool, "PN-BAT-010", "in_stock", "N80")
        .await
        .unwrap();

    let req = BatchLabelRequest {
        pipe_ids: vec![PipeIdentifier { pipe_type: "seamless".into(), pipe_id: pid }],
    };

    let html = LabelService::generate_batch_labels(&pool, &req)
        .await
        .expect("generate_batch_labels must succeed");

    assert!(html.contains("PN-BAT-010"));
    assert!(html.contains("N80"));
}

#[tokio::test]
async fn generate_batch_labels_invalid_pipe_type_in_batch() {
    let pool = common::test_pool().await;

    let req = BatchLabelRequest {
        pipe_ids: vec![PipeIdentifier { pipe_type: "bogus".into(), pipe_id: 1 }],
    };

    let err = LabelService::generate_batch_labels(&pool, &req)
        .await
        .expect_err("must reject invalid pipe type in batch");

    assert!(err.to_string().contains("Invalid pipe_type"));
}

#[tokio::test]
async fn generate_batch_labels_pipe_not_found_in_batch() {
    let pool = common::test_pool().await;

    let req = BatchLabelRequest {
        pipe_ids: vec![PipeIdentifier { pipe_type: "seamless".into(), pipe_id: 99999 }],
    };

    let err = LabelService::generate_batch_labels(&pool, &req)
        .await
        .expect_err("must reject non-existent pipe in batch");

    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn generate_batch_labels_empty_request() {
    let pool = common::test_pool().await;

    let req = BatchLabelRequest { pipe_ids: vec![] };

    let html = LabelService::generate_batch_labels(&pool, &req)
        .await
        .expect("empty batch must succeed");

    assert!(html.contains("</html>"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// generate_quality_tag
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn generate_quality_tag_success() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-QTAG-001", "in_stock", "L80")
        .await
        .unwrap();
    let cert_id = common::seed_quality_cert(&pool, "QC-TAG-001", "seamless", pipe_id, "pass")
        .await
        .unwrap();

    let html = LabelService::generate_quality_tag(&pool, cert_id)
        .await
        .expect("generate_quality_tag must succeed");

    assert!(html.contains("QC-TAG-001"));
    assert!(html.contains("PN-QTAG-001"));
    assert!(html.contains("L80"));
    assert!(html.contains("pass"));
    assert!(html.contains("QUALITY CERTIFICATE"));
}

#[tokio::test]
async fn generate_quality_tag_fail_result_shows_red() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-QTAG-002", "in_stock", "J55")
        .await
        .unwrap();
    let cert_id = common::seed_quality_cert(&pool, "QC-TAG-002", "seamless", pipe_id, "fail")
        .await
        .unwrap();

    let html = LabelService::generate_quality_tag(&pool, cert_id)
        .await
        .expect("generate_quality_tag must succeed");

    assert!(html.contains("fail"));
    assert!(html.contains("#cc0000"));
}

#[tokio::test]
async fn generate_quality_tag_screen_pipe() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_screen_pipe(&pool, "PN-QTAG-SCR", "in_stock", "L80")
        .await
        .unwrap();
    let cert_id = common::seed_quality_cert(&pool, "QC-TAG-SCR", "screen", pipe_id, "pass")
        .await
        .unwrap();

    let html = LabelService::generate_quality_tag(&pool, cert_id)
        .await
        .expect("generate_quality_tag must succeed");

    assert!(html.contains("QC-TAG-SCR"));
    assert!(html.contains("PN-QTAG-SCR"));
    assert!(html.contains("L80"));
}

#[tokio::test]
async fn generate_quality_tag_cert_not_found() {
    let pool = common::test_pool().await;

    let err = LabelService::generate_quality_tag(&pool, 99999)
        .await
        .expect_err("must reject non-existent cert");

    assert!(err.to_string().contains("not found"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// generate_shipping_label
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn generate_shipping_label_seamless() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-SHIP-001", "in_stock", "L80")
        .await
        .unwrap();

    let req = ShippingLabelRequest {
        pipe_type: "seamless".into(),
        pipe_id,
        order_number: Some("SO-2025-001".into()),
        customer_name: Some("Acme Corp".into()),
        destination: Some("Houston, TX".into()),
        po_number: Some("PO-12345".into()),
        ship_date: Some("2025-06-15".into()),
    };

    let html = LabelService::generate_shipping_label(&pool, &req)
        .await
        .expect("generate_shipping_label must succeed");

    assert!(html.contains("PN-SHIP-001"));
    assert!(html.contains("L80"));
    assert!(html.contains("SO-2025-001"));
    assert!(html.contains("Acme Corp"));
    assert!(html.contains("Houston, TX"));
    assert!(html.contains("PO-12345"));
    assert!(html.contains("2025-06-15"));
    assert!(html.contains("SHIPPING LABEL"));
}

#[tokio::test]
async fn generate_shipping_label_screen() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_screen_pipe(&pool, "PN-SHIP-SCR", "in_stock", "N80")
        .await
        .unwrap();

    let req = ShippingLabelRequest {
        pipe_type: "screen".into(),
        pipe_id,
        order_number: None,
        customer_name: None,
        destination: None,
        po_number: None,
        ship_date: None,
    };

    let html = LabelService::generate_shipping_label(&pool, &req)
        .await
        .expect("generate_shipping_label must succeed");

    assert!(html.contains("PN-SHIP-SCR"));
    assert!(html.contains("N80"));
    assert!(html.contains("SHIPPING LABEL"));
}

#[tokio::test]
async fn generate_shipping_label_with_minimal_fields() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-SHIP-002", "in_stock", "J55")
        .await
        .unwrap();

    let req = ShippingLabelRequest {
        pipe_type: "seamless".into(),
        pipe_id,
        order_number: None,
        customer_name: None,
        destination: None,
        po_number: None,
        ship_date: None,
    };

    let html = LabelService::generate_shipping_label(&pool, &req)
        .await
        .expect("generate_shipping_label with minimal fields must succeed");

    assert!(html.contains("PN-SHIP-002"));
    assert!(!html.contains("undefined"));
}

#[tokio::test]
async fn generate_shipping_label_invalid_pipe_type() {
    let pool = common::test_pool().await;

    let req = ShippingLabelRequest {
        pipe_type: "invalid".into(),
        pipe_id: 1,
        order_number: None,
        customer_name: None,
        destination: None,
        po_number: None,
        ship_date: None,
    };

    let err = LabelService::generate_shipping_label(&pool, &req)
        .await
        .expect_err("must reject invalid pipe type");

    assert!(err.to_string().contains("Invalid pipe_type"));
}

#[tokio::test]
async fn generate_shipping_label_pipe_not_found() {
    let pool = common::test_pool().await;

    let req = ShippingLabelRequest {
        pipe_type: "seamless".into(),
        pipe_id: 99999,
        order_number: None,
        customer_name: None,
        destination: None,
        po_number: None,
        ship_date: None,
    };

    let err = LabelService::generate_shipping_label(&pool, &req)
        .await
        .expect_err("must reject non-existent pipe");

    assert!(err.to_string().contains("not found"));
}
