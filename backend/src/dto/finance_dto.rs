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
    pub debit: Option<f64>,
    pub credit: Option<f64>,
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
    pub amount: Option<f64>,
    pub tax_amount: Option<f64>,
    pub due_date: Option<NaiveDate>,
    pub items: Vec<InvoiceItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct InvoiceItemInput {
    pub description: Option<String>,
    pub quantity: Option<f64>,
    pub unit_price: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub invoice_id: Option<i64>,
    pub direction: String,
    pub amount: f64,
    pub method: Option<String>,
    pub reference: Option<String>,
}
