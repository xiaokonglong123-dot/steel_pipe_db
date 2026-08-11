mod common;

use common::test_pool;
use erp_v2::auth::bootstrap_admin;
use erp_v2::error::ErrorCode;
use erp_v2::middleware::auth::AuthUser;
use erp_v2::services::finance_service::{
    self, CreateAccountRequest, CreateInvoiceRequest, CreateJournalEntryRequest,
    CreatePaymentRequest, JournalLineInput,
};
use rust_decimal::Decimal;
use std::str::FromStr;

fn admin() -> AuthUser {
    AuthUser {
        id: 1,
        username: "admin".to_string(),
        display_name: "admin".to_string(),
        permissions: vec![
            "user.manage".to_string(),
            "finance.read".to_string(),
            "finance.write".to_string(),
        ],
    }
}

async fn account(pool: &sqlx::SqlitePool, code: &str, name: &str, account_type: &str) -> i64 {
    finance_service::create_account(
        pool,
        &CreateAccountRequest {
            code: code.to_string(),
            name: name.to_string(),
            parent_id: None,
            account_type: account_type.to_string(),
        },
        &admin(),
    )
    .await
    .unwrap()
    .id
}

async fn balanced_entry(
    pool: &sqlx::SqlitePool,
    debit_account: i64,
    credit_account: i64,
    debit: &str,
    credit: &str,
) -> erp_v2::repos::finance_repo::JournalEntryRow {
    finance_service::create_journal_entry(
        pool,
        &CreateJournalEntryRequest {
            entry_date: "2026-08-10".to_string(),
            description: Some("test".to_string()),
            ref_type: None,
            ref_id: None,
            lines: vec![
                JournalLineInput {
                    account_id: debit_account,
                    debit: debit.to_string(),
                    credit: "0".to_string(),
                    description: None,
                },
                JournalLineInput {
                    account_id: credit_account,
                    debit: "0".to_string(),
                    credit: credit.to_string(),
                    description: None,
                },
            ],
        },
        &admin(),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn create_account_with_valid_type_succeeds() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let row = finance_service::create_account(
        &pool,
        &CreateAccountRequest {
            code: "1001".to_string(),
            name: "现金".to_string(),
            parent_id: None,
            account_type: "asset".to_string(),
        },
        &admin(),
    )
    .await
    .unwrap();
    assert_eq!(row.account_type, "asset");
}

#[tokio::test]
async fn create_journal_entry_balanced_succeeds() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let cash = account(&pool, "1001", "现金", "asset").await;
    let revenue = account(&pool, "6001", "收入", "income").await;
    let row = balanced_entry(&pool, cash, revenue, "100.00", "100.00").await;
    assert_eq!(row.status, "draft");
}

#[tokio::test]
async fn create_journal_entry_unbalanced_returns_16002() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let cash = account(&pool, "1001", "现金", "asset").await;
    let revenue = account(&pool, "6001", "收入", "income").await;
    let result = finance_service::create_journal_entry(
        &pool,
        &CreateJournalEntryRequest {
            entry_date: "2026-08-10".to_string(),
            description: None,
            ref_type: None,
            ref_id: None,
            lines: vec![
                JournalLineInput {
                    account_id: cash,
                    debit: "100".to_string(),
                    credit: "0".to_string(),
                    description: None,
                },
                JournalLineInput {
                    account_id: revenue,
                    debit: "0".to_string(),
                    credit: "99".to_string(),
                    description: None,
                },
            ],
        },
        &admin(),
    )
    .await;
    assert_eq!(result.unwrap_err().code.code(), 16002);
}

#[tokio::test]
async fn journal_line_both_debit_and_credit_rejected() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let cash = account(&pool, "1001", "现金", "asset").await;
    let result = finance_service::create_journal_entry(
        &pool,
        &CreateJournalEntryRequest {
            entry_date: "2026-08-10".to_string(),
            description: None,
            ref_type: None,
            ref_id: None,
            lines: vec![JournalLineInput {
                account_id: cash,
                debit: "50".to_string(),
                credit: "50".to_string(),
                description: None,
            }],
        },
        &admin(),
    )
    .await;
    assert_eq!(result.unwrap_err().code, ErrorCode::Validation);
}

#[tokio::test]
async fn post_journal_entry_changes_status_to_posted() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let cash = account(&pool, "1001", "现金", "asset").await;
    let revenue = account(&pool, "6001", "收入", "income").await;
    let row = balanced_entry(&pool, cash, revenue, "100", "100").await;
    assert_eq!(
        finance_service::post_journal_entry(&pool, row.id)
            .await
            .unwrap()
            .status,
        "posted"
    );
}

#[tokio::test]
async fn trial_balance_after_posted_entries() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let cash = account(&pool, "1001", "现金", "asset").await;
    let revenue = account(&pool, "6001", "收入", "income").await;
    let first = balanced_entry(&pool, cash, revenue, "100", "100").await;
    let second = balanced_entry(&pool, cash, revenue, "30", "30").await;
    finance_service::post_journal_entry(&pool, first.id)
        .await
        .unwrap();
    finance_service::post_journal_entry(&pool, second.id)
        .await
        .unwrap();
    let rows = finance_service::trial_balance(&pool).await.unwrap();
    let debit: Decimal = rows
        .iter()
        .map(|row| Decimal::from_str(&row.total_debit).unwrap())
        .sum();
    let credit: Decimal = rows
        .iter()
        .map(|row| Decimal::from_str(&row.total_credit).unwrap())
        .sum();
    assert_eq!(debit, credit);
}

#[tokio::test]
async fn decimal_precision_test() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let cash = account(&pool, "1001", "现金", "asset").await;
    let revenue = account(&pool, "6001", "收入", "income").await;
    let row = finance_service::create_journal_entry(
        &pool,
        &CreateJournalEntryRequest {
            entry_date: "2026-08-10".to_string(),
            description: None,
            ref_type: None,
            ref_id: None,
            lines: vec![
                JournalLineInput {
                    account_id: cash,
                    debit: "0.1".to_string(),
                    credit: "0".to_string(),
                    description: None,
                },
                JournalLineInput {
                    account_id: cash,
                    debit: "0.2".to_string(),
                    credit: "0".to_string(),
                    description: None,
                },
                JournalLineInput {
                    account_id: revenue,
                    debit: "0".to_string(),
                    credit: "0.3".to_string(),
                    description: None,
                },
            ],
        },
        &admin(),
    )
    .await
    .unwrap();
    assert_eq!(row.status, "draft");
}

#[tokio::test]
async fn create_invoice_and_payment_marks_invoice_paid() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let invoice = finance_service::create_invoice(
        &pool,
        &CreateInvoiceRequest {
            invoice_no: "INV-1".to_string(),
            invoice_date: "2026-08-10".to_string(),
            party_type: "supplier".to_string(),
            party_id: 1,
            amount: "100.00".to_string(),
            ref_type: None,
            ref_id: None,
        },
        &admin(),
    )
    .await
    .unwrap();
    finance_service::create_payment(
        &pool,
        &CreatePaymentRequest {
            payment_no: "PAY-1".to_string(),
            payment_date: "2026-08-10".to_string(),
            supplier_id: None,
            amount: "100.00".to_string(),
            invoice_id: Some(invoice.id),
            method: None,
            notes: None,
        },
        &admin(),
    )
    .await
    .unwrap();
    assert_eq!(
        finance_service::list_invoices(&pool, 1, 20)
            .await
            .unwrap()
            .0[0]
            .status,
        "paid"
    );
}

#[tokio::test]
async fn partial_payment_marks_invoice_partially_paid() {
    let (pool, _dir) = test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let invoice = finance_service::create_invoice(
        &pool,
        &CreateInvoiceRequest {
            invoice_no: "INV-2".to_string(),
            invoice_date: "2026-08-10".to_string(),
            party_type: "supplier".to_string(),
            party_id: 1,
            amount: "100".to_string(),
            ref_type: None,
            ref_id: None,
        },
        &admin(),
    )
    .await
    .unwrap();
    finance_service::create_payment(
        &pool,
        &CreatePaymentRequest {
            payment_no: "PAY-2".to_string(),
            payment_date: "2026-08-10".to_string(),
            supplier_id: None,
            amount: "30".to_string(),
            invoice_id: Some(invoice.id),
            method: None,
            notes: None,
        },
        &admin(),
    )
    .await
    .unwrap();
    assert_eq!(
        finance_service::list_invoices(&pool, 1, 20)
            .await
            .unwrap()
            .0[0]
            .status,
        "partially_paid"
    );
}
