//! Finance DTOs — request payloads for account/journal/invoice/payment endpoints.

use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct JournalDetailInput {
    pub account_id: i64,
    pub debit: Option<rust_decimal::Decimal>,
    pub credit: Option<rust_decimal::Decimal>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJournalEntryRequest {
    pub entry_date: NaiveDate,
    pub description: Option<String>,
    pub details: Vec<JournalDetailInput>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceRequest {
    pub invoice_type: String,
    pub party_id: i64,
    pub order_id: Option<i64>,
    pub amount: Option<rust_decimal::Decimal>,
    pub tax_amount: Option<rust_decimal::Decimal>,
    pub due_date: Option<NaiveDate>,
    pub items: Vec<InvoiceItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct InvoiceItemInput {
    pub description: Option<String>,
    pub quantity: Option<rust_decimal::Decimal>,
    pub unit_price: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub invoice_id: Option<i64>,
    pub direction: String,
    pub amount: rust_decimal::Decimal,
    pub method: Option<String>,
    pub reference: Option<String>,
}
