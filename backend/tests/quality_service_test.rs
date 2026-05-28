//! Integration tests for QualityService.
//!
//! Covers:
//! - Quality cert CRUD (create, read, update, soft delete, listing)
//! - API 5CT grade reference queries (get by code, list all)
//! - Pipe attachment create / delete / list
//!
//! All tests use an in-memory SQLite database with fresh migrations.

mod common;

use steel_pipe_db::dto::common::PaginationParams;
use steel_pipe_db::dto::quality_dto::{
    CreateAttachmentRequest, CreateQualityCertRequest, QualityCertFilterParams,
    UpdateQualityCertRequest,
};
use steel_pipe_db::services::quality_service::QualityService;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// create_cert
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_cert_success() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-QC-001", "in_stock", "L80")
        .await
        .unwrap();

    let dto = CreateQualityCertRequest {
        pipe_type: "seamless".into(),
        pipe_id,
        cert_date: Some("2025-06-01".into()),
        result: Some("pass".into()),
        inspector: Some("John".into()),
        inspection_body: Some("API Lab".into()),
        notes: Some("standard mill cert".into()),
        cert_number: None,
    };

    let cert = QualityService::create_cert(&pool, &dto)
        .await
        .expect("create_cert must succeed");

    assert!(cert.id > 0);
    assert!(cert.cert_number.starts_with("QC-seamless-"));
    assert_eq!(cert.pipe_type, "seamless");
    assert_eq!(cert.pipe_id, pipe_id);
    assert_eq!(cert.result, "pass");
    assert_eq!(cert.inspector.as_deref(), Some("John"));
    assert_eq!(cert.inspection_body.as_deref(), Some("API Lab"));
    assert_eq!(cert.notes.as_deref(), Some("standard mill cert"));
    assert_eq!(cert.cert_date.as_deref(), Some("2025-06-01"));
    assert!(cert.deleted_at.is_none());
}

#[tokio::test]
async fn create_cert_with_explicit_cert_number() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-QC-002", "in_stock", "L80")
        .await
        .unwrap();

    let dto = CreateQualityCertRequest {
        pipe_type: "seamless".into(),
        pipe_id,
        cert_date: Some("2025-06-01".into()),
        result: Some("pass".into()),
        inspector: None,
        inspection_body: None,
        notes: None,
        cert_number: Some("CERT-MANUAL-001".into()),
    };

    // The service replaces the cert_number with a placeholder and auto-generates one,
    // so the provided value is ignored. We test that a valid cert is created anyway.
    let cert = QualityService::create_cert(&pool, &dto)
        .await
        .expect("create_cert must succeed");
    assert!(cert.cert_number.starts_with("QC-seamless-"));
}

#[tokio::test]
async fn create_cert_invalid_result_rejected() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-QC-003", "in_stock", "L80")
        .await
        .unwrap();

    let dto = CreateQualityCertRequest {
        pipe_type: "seamless".into(),
        pipe_id,
        cert_date: None,
        result: Some("invalid_result".into()),
        inspector: None,
        inspection_body: None,
        notes: None,
        cert_number: None,
    };

    let err = QualityService::create_cert(&pool, &dto)
        .await
        .expect_err("must reject invalid result");
    assert!(err.to_string().contains("Invalid result"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// update_cert
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn update_cert_fields() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-QC-010", "in_stock", "L80")
        .await
        .unwrap();
    let cert_id = common::seed_quality_cert(&pool, "QC-UPD-001", "seamless", pipe_id, "pass")
        .await
        .unwrap();

    let update = UpdateQualityCertRequest {
        cert_date: Some("2025-07-15".into()),
        result: Some("fail".into()),
        inspector: Some("Jane".into()),
        inspection_body: Some("Third Party Lab".into()),
        notes: Some("updated notes".into()),
    };

    let updated = QualityService::update_cert(&pool, cert_id, &update)
        .await
        .expect("update_cert must succeed");

    assert_eq!(updated.id, cert_id);
    assert_eq!(updated.result, "fail");
    assert_eq!(updated.inspector.as_deref(), Some("Jane"));
    assert_eq!(
        updated.inspection_body.as_deref(),
        Some("Third Party Lab")
    );
    assert_eq!(updated.notes.as_deref(), Some("updated notes"));
    assert_eq!(updated.cert_date.as_deref(), Some("2025-07-15"));
}

#[tokio::test]
async fn update_cert_partial_fields() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-QC-011", "in_stock", "L80")
        .await
        .unwrap();
    let cert_id = common::seed_quality_cert(&pool, "QC-UPD-002", "seamless", pipe_id, "pass")
        .await
        .unwrap();

    // Only update the result — other fields should stay as seeded
    let update = UpdateQualityCertRequest {
        cert_date: None,
        result: Some("fail".into()),
        inspector: None,
        inspection_body: None,
        notes: None,
    };

    let updated = QualityService::update_cert(&pool, cert_id, &update)
        .await
        .expect("partial update must succeed");

    assert_eq!(updated.result, "fail");
    assert_eq!(updated.inspector.as_deref(), Some("Test Inspector"));
}

#[tokio::test]
async fn update_cert_not_found() {
    let pool = common::test_pool().await;

    let update = UpdateQualityCertRequest {
        cert_date: None,
        result: Some("pass".into()),
        inspector: None,
        inspection_body: None,
        notes: None,
    };

    let err = QualityService::update_cert(&pool, 99999, &update)
        .await
        .expect_err("must fail for non-existent cert");
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn update_cert_invalid_result_rejected() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-QC-012", "in_stock", "L80")
        .await
        .unwrap();
    let cert_id = common::seed_quality_cert(&pool, "QC-UPD-003", "seamless", pipe_id, "pass")
        .await
        .unwrap();

    let update = UpdateQualityCertRequest {
        cert_date: None,
        result: Some("bad".into()),
        inspector: None,
        inspection_body: None,
        notes: None,
    };

    let err = QualityService::update_cert(&pool, cert_id, &update)
        .await
        .expect_err("must reject invalid result");
    assert!(err.to_string().contains("Invalid result"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// delete_cert (soft delete)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn delete_cert_soft_deletes() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-QC-020", "in_stock", "L80")
        .await
        .unwrap();
    let cert_id = common::seed_quality_cert(&pool, "QC-DEL-001", "seamless", pipe_id, "pass")
        .await
        .unwrap();

    QualityService::delete_cert(&pool, cert_id)
        .await
        .expect("delete_cert must succeed");

    // Verify soft-deleted: get_cert should now fail
    let err = QualityService::get_cert(&pool, cert_id)
        .await
        .expect_err("deleted cert must not be found");
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn delete_cert_idempotent_fails_for_already_deleted() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-QC-021", "in_stock", "L80")
        .await
        .unwrap();
    let cert_id = common::seed_quality_cert(&pool, "QC-DEL-002", "seamless", pipe_id, "pass")
        .await
        .unwrap();

    QualityService::delete_cert(&pool, cert_id)
        .await
        .expect("first delete must succeed");

    let err = QualityService::delete_cert(&pool, cert_id)
        .await
        .expect_err("second delete must fail");
    assert!(err.to_string().contains("has been deleted") || err.to_string().contains("not found"));
}

#[tokio::test]
async fn delete_cert_nonexistent_fails() {
    let pool = common::test_pool().await;

    let err = QualityService::delete_cert(&pool, 99999)
        .await
        .expect_err("must fail for non-existent cert");
    assert!(err.to_string().contains("not found"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// get_cert
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn get_cert_existing() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-QC-030", "in_stock", "L80")
        .await
        .unwrap();
    let cert_id = common::seed_quality_cert(&pool, "QC-GET-001", "seamless", pipe_id, "pass")
        .await
        .unwrap();

    let cert = QualityService::get_cert(&pool, cert_id)
        .await
        .expect("get_cert must succeed");

    assert_eq!(cert.id, cert_id);
    assert_eq!(cert.cert_number, "QC-GET-001");
    assert_eq!(cert.result, "pass");
}

#[tokio::test]
async fn get_cert_non_existing() {
    let pool = common::test_pool().await;

    let err = QualityService::get_cert(&pool, 99999)
        .await
        .expect_err("must fail for non-existent cert");
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn get_cert_after_delete_not_found() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-QC-031", "in_stock", "L80")
        .await
        .unwrap();
    let cert_id = common::seed_quality_cert(&pool, "QC-GET-002", "seamless", pipe_id, "pass")
        .await
        .unwrap();

    QualityService::delete_cert(&pool, cert_id)
        .await
        .unwrap();

    let err = QualityService::get_cert(&pool, cert_id)
        .await
        .expect_err("must fail after soft delete");
    assert!(err.to_string().contains("not found"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// list_certs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn list_certs_pagination() {
    let pool = common::test_pool().await;

    let pipe_id_a = common::seed_seamless_pipe(&pool, "PN-LST-001", "in_stock", "L80")
        .await
        .unwrap();
    let pipe_id_b = common::seed_seamless_pipe(&pool, "PN-LST-002", "in_stock", "J55")
        .await
        .unwrap();

    common::seed_quality_cert(&pool, "QC-LST-001", "seamless", pipe_id_a, "pass")
        .await
        .unwrap();
    common::seed_quality_cert(&pool, "QC-LST-002", "seamless", pipe_id_b, "fail")
        .await
        .unwrap();

    let filter = QualityCertFilterParams {
        pipe_type: None,
        pipe_id: None,
        result: None,
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

    let (items, total) = QualityService::list_certs(&pool, &filter, &params)
        .await
        .expect("list_certs must succeed");
    assert_eq!(total, 2);
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn list_certs_filter_by_pipe_type() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-LST-010", "in_stock", "L80")
        .await
        .unwrap();

    common::seed_quality_cert(&pool, "QC-LST-010", "seamless", pipe_id, "pass")
        .await
        .unwrap();

    let filter = QualityCertFilterParams {
        pipe_type: Some("seamless".into()),
        pipe_id: None,
        result: None,
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

    let (items, total) = QualityService::list_certs(&pool, &filter, &params)
        .await
        .expect("list_certs must succeed");
    assert_eq!(total, 1);
    assert_eq!(items[0].pipe_type, "seamless");
}

#[tokio::test]
async fn list_certs_filter_by_pipe_id() {
    let pool = common::test_pool().await;

    let pipe_id_a = common::seed_seamless_pipe(&pool, "PN-LST-020", "in_stock", "L80")
        .await
        .unwrap();
    let pipe_id_b = common::seed_seamless_pipe(&pool, "PN-LST-021", "in_stock", "J55")
        .await
        .unwrap();

    common::seed_quality_cert(&pool, "QC-LST-020", "seamless", pipe_id_a, "pass")
        .await
        .unwrap();
    common::seed_quality_cert(&pool, "QC-LST-021", "seamless", pipe_id_b, "fail")
        .await
        .unwrap();

    let filter = QualityCertFilterParams {
        pipe_type: None,
        pipe_id: Some(pipe_id_a),
        result: None,
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

    let (items, total) = QualityService::list_certs(&pool, &filter, &params)
        .await
        .expect("list_certs must succeed");
    assert_eq!(total, 1);
    assert_eq!(items[0].pipe_id, pipe_id_a);
}

#[tokio::test]
async fn list_certs_filter_by_result() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-LST-030", "in_stock", "L80")
        .await
        .unwrap();

    common::seed_quality_cert(&pool, "QC-LST-030", "seamless", pipe_id, "pass")
        .await
        .unwrap();
    common::seed_quality_cert(&pool, "QC-LST-031", "seamless", pipe_id, "fail")
        .await
        .unwrap();

    let filter = QualityCertFilterParams {
        pipe_type: None,
        pipe_id: None,
        result: Some("fail".into()),
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

    let (items, total) = QualityService::list_certs(&pool, &filter, &params)
        .await
        .expect("list_certs must succeed");
    assert_eq!(total, 1);
    assert_eq!(items[0].result, "fail");
}

#[tokio::test]
async fn list_certs_empty_when_no_match() {
    let pool = common::test_pool().await;

    let filter = QualityCertFilterParams {
        pipe_type: Some("nonexistent".into()),
        pipe_id: None,
        result: None,
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

    let (items, total) = QualityService::list_certs(&pool, &filter, &params)
        .await
        .expect("list_certs must succeed");
    assert_eq!(total, 0);
    assert!(items.is_empty());
}

#[tokio::test]
async fn list_certs_pagination_page_size() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-LST-040", "in_stock", "L80")
        .await
        .unwrap();

    common::seed_quality_cert(&pool, "QC-LST-040", "seamless", pipe_id, "pass")
        .await
        .unwrap();
    common::seed_quality_cert(&pool, "QC-LST-041", "seamless", pipe_id, "pass")
        .await
        .unwrap();
    common::seed_quality_cert(&pool, "QC-LST-042", "seamless", pipe_id, "pass")
        .await
        .unwrap();

    let filter = QualityCertFilterParams {
        pipe_type: None,
        pipe_id: None,
        result: None,
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

    let (items, total) = QualityService::list_certs(&pool, &filter, &params)
        .await
        .expect("list_certs must succeed");
    assert_eq!(total, 3);
    assert_eq!(items.len(), 2);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// get_grade
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn get_grade_by_code() {
    let pool = common::test_pool().await;

    common::seed_api5ct_grade_ref(&pool, "CUSTOM1")
        .await
        .unwrap();

    let grade = QualityService::get_grade(&pool, "CUSTOM1")
        .await
        .expect("get_grade must succeed");

    assert_eq!(grade.grade, "CUSTOM1");
    assert_eq!(grade.yield_strength_min, Some(379.0));
    assert_eq!(grade.yield_strength_max, Some(552.0));
    assert_eq!(grade.tensile_strength_min, Some(517.0));
    assert!(grade.hardness_max.is_some());
    assert!(grade.carbon_content_max.is_some());
}

#[tokio::test]
async fn get_grade_non_existing_returns_not_found() {
    let pool = common::test_pool().await;

    let err = QualityService::get_grade(&pool, "ZZZ999")
        .await
        .expect_err("must fail for non-existent grade");
    assert!(err.to_string().contains("not found"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// list_grades
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn list_grades_all() {
    let pool = common::test_pool().await;

    common::seed_api5ct_grade_ref(&pool, "CUSTOM1")
        .await
        .unwrap();
    common::seed_api5ct_grade_ref(&pool, "CUSTOM2")
        .await
        .unwrap();
    common::seed_api5ct_grade_ref(&pool, "CUSTOM3")
        .await
        .unwrap();

    let grades = QualityService::list_grades(&pool)
        .await
        .expect("list_grades must succeed");

    // 9 seeded by migration 010 + 3 custom = 12
    assert_eq!(grades.len(), 12);
    let names: Vec<&str> = grades.iter().map(|g| g.grade.as_str()).collect();
    assert!(names.contains(&"CUSTOM1"));
    assert!(names.contains(&"CUSTOM2"));
    assert!(names.contains(&"CUSTOM3"));
    assert!(names.contains(&"L80"));
    assert!(names.contains(&"J55"));
}

#[tokio::test]
async fn list_grades_returns_seeded_defaults() {
    let pool = common::test_pool().await;

    let grades = QualityService::list_grades(&pool)
        .await
        .expect("list_grades must succeed");
    // Migration 010 seeds 9 standard API 5CT grades
    assert!(!grades.is_empty(), "should have seeded grade data");
    assert_eq!(grades.len(), 9, "migration 010 seeds 9 grades");
    let names: Vec<&str> = grades.iter().map(|g| g.grade.as_str()).collect();
    assert!(names.contains(&"L80"));
    assert!(names.contains(&"J55"));
    assert!(names.contains(&"N80"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// create_attachment
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_attachment_success() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-ATT-001", "in_stock", "L80")
        .await
        .unwrap();

    let dto = CreateAttachmentRequest {
        pipe_type: "seamless".into(),
        pipe_id,
        file_name: "cert_l80.pdf".into(),
        file_path: "/storage/certs/cert_l80.pdf".into(),
        file_size: Some(2048),
        content_type: Some("application/pdf".into()),
        uploaded_by: Some(1),
    };

    let attachment = QualityService::create_attachment(&pool, &dto)
        .await
        .expect("create_attachment must succeed");

    assert!(attachment.id > 0);
    assert_eq!(attachment.pipe_type, "seamless");
    assert_eq!(attachment.pipe_id, pipe_id);
    assert_eq!(attachment.file_name, "cert_l80.pdf");
    assert_eq!(attachment.file_path, "/storage/certs/cert_l80.pdf");
    assert_eq!(attachment.file_size, Some(2048));
    assert_eq!(attachment.content_type, Some("application/pdf".into()));
    assert_eq!(attachment.uploaded_by, Some(1));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// delete_attachment
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn delete_attachment_success() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-ATT-010", "in_stock", "L80")
        .await
        .unwrap();
    let att_id = common::seed_pipe_attachment(&pool, "seamless", pipe_id, "doc.pdf")
        .await
        .unwrap();

    QualityService::delete_attachment(&pool, att_id)
        .await
        .expect("delete_attachment must succeed");

    // Verify: listing attachments for that pipe should return no results
    let list = QualityService::list_attachments(&pool, "seamless", pipe_id)
        .await
        .expect("list_attachments must succeed");
    assert!(list.is_empty());
}

#[tokio::test]
async fn delete_attachment_nonexistent_fails() {
    let pool = common::test_pool().await;

    let err = QualityService::delete_attachment(&pool, 99999)
        .await
        .expect_err("must fail for non-existent attachment");
    assert!(err.to_string().contains("not found"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// list_attachments
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn list_attachments_by_pipe_id() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-ATT-020", "in_stock", "L80")
        .await
        .unwrap();

    common::seed_pipe_attachment(&pool, "seamless", pipe_id, "doc_a.pdf")
        .await
        .unwrap();
    common::seed_pipe_attachment(&pool, "seamless", pipe_id, "doc_b.pdf")
        .await
        .unwrap();

    let list = QualityService::list_attachments(&pool, "seamless", pipe_id)
        .await
        .expect("list_attachments must succeed");

    assert_eq!(list.len(), 2);
    let names: Vec<&str> = list.iter().map(|a| a.file_name.as_str()).collect();
    assert!(names.contains(&"doc_a.pdf"));
    assert!(names.contains(&"doc_b.pdf"));
}

#[tokio::test]
async fn list_attachments_by_pipe_id_filters_correctly() {
    let pool = common::test_pool().await;

    let pipe_id_a = common::seed_seamless_pipe(&pool, "PN-ATT-030", "in_stock", "L80")
        .await
        .unwrap();
    let pipe_id_b = common::seed_seamless_pipe(&pool, "PN-ATT-031", "in_stock", "J55")
        .await
        .unwrap();

    common::seed_pipe_attachment(&pool, "seamless", pipe_id_a, "for_pipe_a.pdf")
        .await
        .unwrap();
    common::seed_pipe_attachment(&pool, "seamless", pipe_id_b, "for_pipe_b.pdf")
        .await
        .unwrap();

    let list_a = QualityService::list_attachments(&pool, "seamless", pipe_id_a)
        .await
        .expect("list_attachments must succeed");
    assert_eq!(list_a.len(), 1);
    assert_eq!(list_a[0].file_name, "for_pipe_a.pdf");

    let list_b = QualityService::list_attachments(&pool, "seamless", pipe_id_b)
        .await
        .expect("list_attachments must succeed");
    assert_eq!(list_b.len(), 1);
    assert_eq!(list_b[0].file_name, "for_pipe_b.pdf");
}

#[tokio::test]
async fn list_attachments_empty_when_none() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-ATT-040", "in_stock", "L80")
        .await
        .unwrap();

    let list = QualityService::list_attachments(&pool, "seamless", pipe_id)
        .await
        .expect("list_attachments must succeed");
    assert!(list.is_empty());
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Full cert lifecycle: create → get → list → update → soft delete
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn cert_lifecycle_create_get_list_update_delete() {
    let pool = common::test_pool().await;

    // ── Create pipe ──
    let pipe_id = common::seed_seamless_pipe(&pool, "PN-LIFE-001", "in_stock", "L80")
        .await
        .unwrap();

    // ── Create cert ──
    let create = CreateQualityCertRequest {
        pipe_type: "seamless".into(),
        pipe_id,
        cert_date: Some("2025-06-01".into()),
        result: Some("pass".into()),
        inspector: Some("Inspector A".into()),
        inspection_body: Some("Lab X".into()),
        notes: Some("initial cert".into()),
        cert_number: None,
    };
    let cert = QualityService::create_cert(&pool, &create)
        .await
        .expect("create_cert must succeed");
    let cert_id = cert.id;
    assert_eq!(cert.result, "pass");

    // ── Get cert ──
    let fetched = QualityService::get_cert(&pool, cert_id)
        .await
        .expect("get_cert must succeed");
    assert_eq!(fetched.id, cert_id);
    assert_eq!(fetched.notes.as_deref(), Some("initial cert"));

    // ── List certs — should include our cert ──
    let filter = QualityCertFilterParams {
        pipe_type: None,
        pipe_id: None,
        result: None,
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
    let (items, total) = QualityService::list_certs(&pool, &filter, &params)
        .await
        .expect("list_certs must succeed");
    assert!(total >= 1);
    assert!(items.iter().any(|c| c.id == cert_id));

    // ── Update cert ──
    let update = UpdateQualityCertRequest {
        cert_date: Some("2025-06-15".into()),
        result: Some("fail".into()),
        inspector: None,
        inspection_body: None,
        notes: Some("re-test required".into()),
    };
    let updated = QualityService::update_cert(&pool, cert_id, &update)
        .await
        .expect("update_cert must succeed");
    assert_eq!(updated.result, "fail");
    assert_eq!(updated.notes.as_deref(), Some("re-test required"));

    // ── Soft delete cert ──
    QualityService::delete_cert(&pool, cert_id)
        .await
        .expect("delete_cert must succeed");

    let err = QualityService::get_cert(&pool, cert_id)
        .await
        .expect_err("deleted cert should not be found");
    assert!(err.to_string().contains("not found"));
}
