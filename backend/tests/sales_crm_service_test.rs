//! Sales CRM integration tests — shipments, quotes, quote→order conversion.

mod common;

use rust_decimal_macros::dec;
use steel_pipe_db::dto::sales_crm_dto::{CreateSalesQuoteRequest, CreateShipmentRequest};
use steel_pipe_db::sales_crm::services::SalesCrmService;

#[tokio::test]
async fn shipment_lifecycle() {
    let pool = common::test_pool().await;
    let shipment = SalesCrmService::create_shipment(
        &pool,
        1,
        Some(1),
        &CreateShipmentRequest {
            sales_order_id: 1,
            carrier: Some("顺丰".into()),
            tracking_no: Some("SF123".into()),
            items: vec![serde_json::json!({"pipe_number": "PN-9", "quantity": 5})],
            notes: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(shipment.status, "pending");

    let shipped = SalesCrmService::update_shipment_status(&pool, 1, shipment.id, "shipped").await.unwrap();
    assert_eq!(shipped.status, "shipped");
    assert!(shipped.shipped_at.is_some(), "shipped_at must be stamped on ship");

    let delivered = SalesCrmService::update_shipment_status(&pool, 1, shipment.id, "delivered").await.unwrap();
    assert_eq!(delivered.status, "delivered");
}

#[tokio::test]
async fn invalid_shipment_status_rejected() {
    let pool = common::test_pool().await;
    let shipment = SalesCrmService::create_shipment(
        &pool,
        1,
        None,
        &CreateShipmentRequest {
            sales_order_id: 1,
            carrier: None,
            tracking_no: None,
            items: vec![serde_json::json!({"quantity": 1})],
            notes: None,
        },
    )
    .await
    .unwrap();
    let err = SalesCrmService::update_shipment_status(&pool, 1, shipment.id, "lost").await;
    assert!(err.is_err(), "invalid status must be rejected");
}

#[tokio::test]
async fn quote_requires_confirmed_to_convert() {
    let pool = common::test_pool().await;
    // Customer 1 must exist for FK (customers table) — seed it.
    sqlx::query(
        "INSERT INTO customers (customer_code, name) VALUES ('C-ATP-1', '测试客户') ON CONFLICT DO NOTHING",
    )
    .execute(&pool)
    .await
    .unwrap();

    let quote = SalesCrmService::create_quote(
        &pool,
        1,
        &CreateSalesQuoteRequest {
            customer_id: 1,
            valid_until: None,
            total_amount: dec!(8800),
            items: vec![serde_json::json!({"pipe_type": "seamless", "grade": "J55", "quantity": 10, "unit_price": 880})],
            notes: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(quote.status, "draft");

    // Converting a draft quote must fail.
    let err = SalesCrmService::convert_quote(&pool, 1, quote.id).await;
    assert!(err.is_err(), "draft quote must not convert");

    // Confirm then convert.
    let confirmed = SalesCrmService::update_quote_status(&pool, 1, quote.id, "confirmed").await.unwrap();
    assert_eq!(confirmed.status, "confirmed");
    let order_id = SalesCrmService::convert_quote(&pool, 1, quote.id).await.unwrap();
    assert!(order_id > 0, "conversion must return a sales order id");

    // The order must exist in sales_orders.
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sales_orders WHERE id = $1 AND customer_id = 1")
        .bind(order_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 1, "converted order must exist");

    // Quote now converted; second convert must fail.
    let err2 = SalesCrmService::convert_quote(&pool, 1, quote.id).await;
    assert!(err2.is_err(), "converted quote must not convert twice");
}

#[tokio::test]
async fn customer_credit_snapshot() {
    let pool = common::test_pool().await;
    let credit = SalesCrmService::customer_credit(&pool, 1, 1).await.unwrap();
    assert_eq!(credit.customer_id, 1);
    // Fresh data → zero totals.
    assert_eq!(credit.open_invoice_total, dec!(0));
    assert_eq!(credit.lifetime_sales, dec!(0));
}
