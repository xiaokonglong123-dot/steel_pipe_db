//! Finance integration tests — accounts, journal entries (balance rule),
//! invoices (state machine), payments (auto-settle).

mod common;

use erp_server::dto::finance_dto::{
    CreateAccountRequest, CreateInvoiceRequest, CreateJournalEntryRequest, CreatePaymentRequest,
    InvoiceItemInput, JournalDetailInput,
};
use erp_server::finance::services::FinanceService;

#[tokio::test]
async fn account_crud() {
    let pool = common::test_pool().await;
    let acc = FinanceService::create_account(
        &pool,
        1,
        &CreateAccountRequest {
            code: "1001".into(),
            name: "库存现金".into(),
            account_type: "asset".into(),
            parent_id: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(acc.code, "1001");

    // Duplicate code rejected.
    let dup = FinanceService::create_account(
        &pool,
        1,
        &CreateAccountRequest {
            code: "1001".into(),
            name: "重复".into(),
            account_type: "asset".into(),
            parent_id: None,
        },
    )
    .await;
    assert!(dup.is_err(), "duplicate account code must be rejected");
}

#[tokio::test]
async fn journal_entry_balance_rule() {
    let pool = common::test_pool().await;
    let a1 = FinanceService::create_account(&pool, 1, &CreateAccountRequest { code: "1001".into(), name: "现金".into(), account_type: "asset".into(), parent_id: None }).await.unwrap();
    let a2 = FinanceService::create_account(&pool, 1, &CreateAccountRequest { code: "6001".into(), name: "收入".into(), account_type: "revenue".into(), parent_id: None }).await.unwrap();

    // Unbalanced → rejected.
    let bad = FinanceService::create_journal_entry(
        &pool,
        1,
        &CreateJournalEntryRequest {
            entry_date: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            description: Some("不平衡".into()),
            details: vec![JournalDetailInput { account_id: a1.id, debit: Some(100.0), credit: None, description: None }],
        },
        None,
    )
    .await;
    assert!(bad.is_err(), "unbalanced entry must be rejected");

    // Balanced → posted.
    let ok = FinanceService::create_journal_entry(
        &pool,
        1,
        &CreateJournalEntryRequest {
            entry_date: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            description: Some("销售".into()),
            details: vec![
                JournalDetailInput { account_id: a1.id, debit: Some(500.0), credit: None, description: None },
                JournalDetailInput { account_id: a2.id, debit: None, credit: Some(500.0), description: None },
            ],
        },
        None,
    )
    .await
    .unwrap();
    assert_eq!(ok.status, "posted");
}

#[tokio::test]
async fn trial_balance_after_posting() {
    let pool = common::test_pool().await;
    let a1 = FinanceService::create_account(&pool, 1, &CreateAccountRequest { code: "1001".into(), name: "现金".into(), account_type: "asset".into(), parent_id: None }).await.unwrap();
    let a2 = FinanceService::create_account(&pool, 1, &CreateAccountRequest { code: "6001".into(), name: "收入".into(), account_type: "revenue".into(), parent_id: None }).await.unwrap();
    FinanceService::create_journal_entry(
        &pool,
        1,
        &CreateJournalEntryRequest {
            entry_date: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            description: None,
            details: vec![
                JournalDetailInput { account_id: a1.id, debit: Some(800.0), credit: None, description: None },
                JournalDetailInput { account_id: a2.id, debit: None, credit: Some(800.0), description: None },
            ],
        },
        None,
    )
    .await
    .unwrap();

    let tb = FinanceService::trial_balance(&pool, 1).await.unwrap();
    assert_eq!(tb.len(), 2);
    let cash = tb.iter().find(|r| r.code == "1001").unwrap();
    let revenue = tb.iter().find(|r| r.code == "6001").unwrap();
    assert_eq!(cash.debit, 800.0);
    assert_eq!(revenue.credit, 800.0);
}

#[tokio::test]
async fn invoice_lifecycle_and_auto_settle() {
    let pool = common::test_pool().await;
    let inv = FinanceService::create_invoice(
        &pool,
        1,
        &CreateInvoiceRequest {
            invoice_type: "sales".into(),
            party_id: 1,
            order_id: None,
            amount: None,
            tax_amount: Some(13.0),
            due_date: None,
            items: vec![InvoiceItemInput {
                description: Some("商品A".into()),
                quantity: Some(2.0),
                unit_price: Some(500.0),
            }],
        },
    )
    .await
    .unwrap();
    // 2 × 500 = 1000 + 13 tax = 1013.
    assert_eq!(inv.total_amount, 1013.0);
    assert_eq!(inv.status, "draft");

    // Confirm.
    let confirmed = FinanceService::confirm_invoice(&pool, 1, inv.id).await.unwrap();
    assert_eq!(confirmed.status, "confirmed");

    // Void after confirm is allowed; void a paid invoice is not.
    let _ = FinanceService::void_invoice(&pool, 1, inv.id).await.unwrap();

    // New invoice → pay in full → auto paid.
    let inv2 = FinanceService::create_invoice(
        &pool,
        1,
        &CreateInvoiceRequest {
            invoice_type: "sales".into(),
            party_id: 1,
            order_id: None,
            amount: Some(300.0),
            tax_amount: None,
            due_date: None,
            items: vec![],
        },
    )
    .await
    .unwrap();
    FinanceService::confirm_invoice(&pool, 1, inv2.id).await.unwrap();
    let pay = FinanceService::create_payment(
        &pool,
        1,
        &CreatePaymentRequest {
            invoice_id: Some(inv2.id),
            direction: "in".into(),
            amount: 300.0,
            method: Some("bank_transfer".into()),
            reference: Some("REF-1".into()),
        },
        None,
    )
    .await
    .unwrap();
    assert_eq!(pay.direction, "in");
    let after = FinanceService::get_invoice(&pool, 1, inv2.id).await.unwrap().0;
    assert_eq!(after.status, "paid", "fully paid invoice must auto-transition to paid");
}

#[tokio::test]
async fn partial_payment_keeps_invoice_confirmed() {
    let pool = common::test_pool().await;
    let inv = FinanceService::create_invoice(
        &pool,
        1,
        &CreateInvoiceRequest {
            invoice_type: "sales".into(),
            party_id: 1,
            order_id: None,
            amount: Some(500.0),
            tax_amount: None,
            due_date: None,
            items: vec![],
        },
    )
    .await
    .unwrap();
    FinanceService::confirm_invoice(&pool, 1, inv.id).await.unwrap();
    FinanceService::create_payment(
        &pool,
        1,
        &CreatePaymentRequest {
            invoice_id: Some(inv.id),
            direction: "in".into(),
            amount: 200.0,
            method: None,
            reference: None,
        },
        None,
    )
    .await
    .unwrap();
    let after = FinanceService::get_invoice(&pool, 1, inv.id).await.unwrap().0;
    assert_eq!(after.status, "confirmed", "partial payment must not settle the invoice");
}
