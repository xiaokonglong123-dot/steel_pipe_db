//! Portal integration tests — account creation, login, PO accept, SO ack.

mod common;

use steel_pipe_db::dto::portal_dto::{AcceptPurchaseRequest, CreatePortalAccountRequest, PortalLoginRequest};
use steel_pipe_db::middleware::auth::JwtSecret;
use steel_pipe_db::portal::services::PortalService;

#[tokio::test]
async fn portal_account_and_login() {
    let pool = common::test_pool().await;
    let account = PortalService::create_account(
        &pool, 1,
        &CreatePortalAccountRequest {
            party_type: "supplier".into(),
            party_id: 1,
            username: "supplier_abc".into(),
            password: "secret123".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(account.party_type, "supplier");
    assert_ne!(account.password_hash, "secret123", "password must be hashed");

    // Login with correct password → JWT.
    let secret = JwtSecret("test-secret-key-for-portal-tests-0123456789".to_string());
    let (token, logged_in) = PortalService::login(
        &pool,
        &PortalLoginRequest { username: "supplier_abc".into(), password: "secret123".into() },
        &secret,
    )
    .await
    .unwrap();
    assert!(token.len() > 40);
    assert_eq!(logged_in.party_id, 1);

    // Wrong password rejected.
    let err = PortalService::login(
        &pool,
        &PortalLoginRequest { username: "supplier_abc".into(), password: "wrong".into() },
        &secret,
    )
    .await;
    assert!(err.is_err(), "wrong password must be rejected");
}

#[tokio::test]
async fn duplicate_username_rejected() {
    let pool = common::test_pool().await;
    PortalService::create_account(
        &pool, 1,
        &CreatePortalAccountRequest { party_type: "customer".into(), party_id: 1, username: "dup_user".into(), password: "x".into() },
    )
    .await
    .unwrap();
    let err = PortalService::create_account(
        &pool, 1,
        &CreatePortalAccountRequest { party_type: "supplier".into(), party_id: 2, username: "dup_user".into(), password: "y".into() },
    )
    .await;
    assert!(err.is_err(), "duplicate username must be rejected");
}

#[tokio::test]
async fn supplier_sees_and_accepts_own_po() {
    let pool = common::test_pool().await;
    // Seed a purchase order for supplier 5.
    sqlx::query(
        "INSERT INTO purchase_orders (order_no, supplier_id, order_date, status, total_amount) \
         VALUES ('PO-PORTAL-1', 5, NOW(), 'pending', 9999)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let orders = PortalService::supplier_purchases(&pool, 1, 5).await.unwrap();
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].order_no, "PO-PORTAL-1");

    // Supplier 5 accepts → event recorded + status approved.
    let event = PortalService::accept_purchase(&pool, 1, 5, orders[0].id, &AcceptPurchaseRequest { notes: Some("确认接单".into()) })
        .await
        .unwrap();
    assert_eq!(event.event_type, "po_accepted");

    let status: String = sqlx::query_scalar("SELECT status FROM purchase_orders WHERE order_no = 'PO-PORTAL-1'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "approved");

    // Another supplier cannot accept it.
    let err = PortalService::accept_purchase(&pool, 1, 99, orders[0].id, &AcceptPurchaseRequest { notes: None }).await;
    assert!(err.is_err(), "foreign supplier must not accept the PO");
}

#[tokio::test]
async fn customer_acknowledges_sales_order() {
    let pool = common::test_pool().await;
    sqlx::query(
        "INSERT INTO sales_orders (order_no, customer_id, order_date, status, total_amount) \
         VALUES ('SO-PORTAL-1', 3, NOW(), 'approved', 5000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let orders = PortalService::customer_sales(&pool, 1, 3).await.unwrap();
    assert_eq!(orders.len(), 1);
    let event = PortalService::acknowledge_sales(&pool, 1, 3, orders[0].id).await.unwrap();
    assert_eq!(event.event_type, "so_acknowledged");

    let events = PortalService::events(&pool, 1, "customer", 3).await.unwrap();
    assert_eq!(events.len(), 1);
}
