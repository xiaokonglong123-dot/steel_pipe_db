//! Procurement integration tests — requisitions, receipts, quotes, scorecard.

mod common;

use rust_decimal_macros::dec;
use steel_pipe_db::dto::procurement_dto::{
    CreateQuoteRequest, CreateReceiptRequest, CreateRequisitionRequest, ReceiptItemInput,
    UpdateQuoteStatusRequest,
};
use steel_pipe_db::procurement::services::ProcurementService;

#[tokio::test]
async fn requisition_lifecycle() {
    let pool = common::test_pool().await;
    let req = ProcurementService::create_requisition(
        &pool,
        1,
        Some(1),
        &CreateRequisitionRequest {
            title: "采购钢管".into(),
            department_id: None,
            expected_date: None,
            items: vec![serde_json::json!({"pipe_type": "seamless", "quantity": 10})],
            notes: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(req.status, "draft");

    let submitted = ProcurementService::update_requisition_status(&pool, 1, req.id, "submitted").await.unwrap();
    assert_eq!(submitted.status, "submitted");
    let approved = ProcurementService::update_requisition_status(&pool, 1, req.id, "approved").await.unwrap();
    assert_eq!(approved.status, "approved");

    // Approved → reject is blocked.
    let err = ProcurementService::update_requisition_status(&pool, 1, req.id, "rejected").await;
    assert!(err.is_err(), "approved requisition must not change status");
}

#[tokio::test]
async fn goods_receipt_creation() {
    let pool = common::test_pool().await;
    let receipt = ProcurementService::create_receipt(
        &pool,
        1,
        Some(1),
        &CreateReceiptRequest {
            purchase_order_id: 1,
            notes: Some("第一批到货".into()),
            items: vec![ReceiptItemInput {
                pipe_id: None,
                pipe_number: Some("PN-001".into()),
                quantity: dec!(20),
                remark: None,
            }],
        },
    )
    .await
    .unwrap();
    assert_eq!(receipt.status, "received");
    let (got, items) = ProcurementService::get_receipt(&pool, 1, receipt.id).await.unwrap();
    assert_eq!(got.receipt_no, receipt.receipt_no);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].pipe_number.as_deref(), Some("PN-001"));
}

#[tokio::test]
async fn empty_receipt_rejected() {
    let pool = common::test_pool().await;
    let err = ProcurementService::create_receipt(
        &pool,
        1,
        None,
        &CreateReceiptRequest {
            purchase_order_id: 1,
            notes: None,
            items: vec![],
        },
    )
    .await;
    assert!(err.is_err(), "receipt without items must be rejected");
}

#[tokio::test]
async fn quote_status_transitions() {
    let pool = common::test_pool().await;
    let quote = ProcurementService::create_quote(
        &pool,
        1,
        &CreateQuoteRequest {
            supplier_id: 1,
            title: Some("无缝管报价".into()),
            valid_until: None,
            total_amount: dec!(50000),
            items: vec![serde_json::json!({"grade": "J55", "quantity": 100})],
            notes: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(quote.status, "draft");

    let sent = ProcurementService::update_quote_status(
        &pool,
        1,
        quote.id,
        &UpdateQuoteStatusRequest { status: "sent".into() },
    )
    .await
    .unwrap();
    assert_eq!(sent.status, "sent");

    let accepted = ProcurementService::update_quote_status(
        &pool,
        1,
        quote.id,
        &UpdateQuoteStatusRequest { status: "accepted".into() },
    )
    .await
    .unwrap();
    assert_eq!(accepted.status, "accepted");
}

#[tokio::test]
async fn supplier_scorecard_aggregates() {
    let pool = common::test_pool().await;
    // Insert a purchase order directly for scorecard aggregation.
    sqlx::query(
        "INSERT INTO purchase_orders (order_no, supplier_id, order_date, total_amount, status) \
         VALUES ('PO-SC-1', 5, NOW(), $1, 'approved')",
    )
    .bind(dec!(12345))
    .execute(&pool)
    .await
    .unwrap();

    let card = ProcurementService::supplier_scorecard(&pool, 1, 5).await.unwrap();
    assert_eq!(card.order_count, 1);
    assert_eq!(card.order_total, dec!(12345));
}
