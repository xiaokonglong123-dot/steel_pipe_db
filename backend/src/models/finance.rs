//! Finance row models — sqlx `FromRow` structs mirroring `028_create_finance.sql`.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Account {
    pub id: i64,
    pub tenant_id: i64,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub parent_id: Option<i64>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct JournalEntry {
    pub id: i64,
    pub tenant_id: i64,
    pub entry_no: String,
    pub entry_date: NaiveDate,
    pub description: Option<String>,
    pub status: String,
    pub currency: String,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub posted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct JournalEntryDetail {
    pub id: i64,
    pub entry_id: i64,
    pub account_id: i64,
    pub debit: rust_decimal::Decimal,
    pub credit: rust_decimal::Decimal,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct FinanceInvoice {
    pub id: i64,
    pub tenant_id: i64,
    pub invoice_no: String,
    pub invoice_type: String,
    pub party_id: i64,
    pub order_id: Option<i64>,
    pub amount: rust_decimal::Decimal,
    pub tax_amount: rust_decimal::Decimal,
    pub total_amount: rust_decimal::Decimal,
    pub status: String,
    pub due_date: Option<NaiveDate>,
    pub issued_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct FinanceInvoiceItem {
    pub id: i64,
    pub invoice_id: i64,
    pub description: Option<String>,
    pub quantity: rust_decimal::Decimal,
    pub unit_price: rust_decimal::Decimal,
    pub amount: rust_decimal::Decimal,
}

#[derive(Debug, Serialize, FromRow)]
pub struct FinancePayment {
    pub id: i64,
    pub tenant_id: i64,
    pub payment_no: String,
    pub invoice_id: Option<i64>,
    pub direction: String,
    pub amount: rust_decimal::Decimal,
    pub method: String,
    pub paid_at: DateTime<Utc>,
    pub reference: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
}

/// Trial balance row: account + total debit/credit.
#[derive(Debug, Serialize, FromRow)]
pub struct TrialBalanceRow {
    pub code: String,
    pub name: String,
    pub debit: rust_decimal::Decimal,
    pub credit: rust_decimal::Decimal,
}
