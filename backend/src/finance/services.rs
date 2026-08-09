//! Finance services — accounts, journal entries (with balance validation),
//! invoices (AR/AP state machine), payments (auto-settle invoices).

use sqlx::SqlitePool;

use crate::dto::finance_dto::{
    CreateAccountRequest, CreateInvoiceRequest, CreateJournalEntryRequest, CreatePaymentRequest,
    UpdateAccountRequest,
};
use crate::error::AppError;
use crate::finance::repos::{AccountRepo, InvoiceRepo, JournalEntryRepo, PaymentRepo};
use crate::models::finance::{
    Account, FinanceInvoice, FinanceInvoiceItem, FinancePayment, JournalEntry, JournalEntryDetail,
    TrialBalanceRow,
};

pub struct FinanceService;

impl FinanceService {
    // -----------------------------------------------------------------------
    // Chart of accounts
    // -----------------------------------------------------------------------

    pub async fn list_accounts(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<Account>, AppError> {
        AccountRepo::list(pool, tenant_id).await.map_err(AppError::from)
    }

    pub async fn create_account(
        pool: &SqlitePool,
        tenant_id: i64,
        dto: &CreateAccountRequest,
    ) -> Result<Account, AppError> {
        if dto.code.trim().is_empty() || dto.name.trim().is_empty() {
            return Err(AppError::Validation("Code and name are required".into()));
        }
        if !matches!(dto.account_type.as_str(), "asset" | "liability" | "equity" | "revenue" | "expense") {
            return Err(AppError::Validation(format!(
                "Invalid account type: {}",
                dto.account_type
            )));
        }
        if AccountRepo::find_by_code(pool, tenant_id, &dto.code).await?.is_some() {
            return Err(AppError::Validation(format!("Account code '{}' already exists", dto.code)));
        }
        AccountRepo::create(pool, tenant_id, dto.code.trim(), dto.name.trim(), &dto.account_type, dto.parent_id)
            .await
            .map_err(AppError::from)
    }

    pub async fn update_account(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        dto: &UpdateAccountRequest,
    ) -> Result<Account, AppError> {
        AccountRepo::update(pool, tenant_id, id, dto.name.as_deref(), dto.is_active)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Account not found: {}", id)))
    }

    // -----------------------------------------------------------------------
    // Journal entries
    // -----------------------------------------------------------------------

    /// Create a journal entry; validates that debit total == credit total.
    pub async fn create_journal_entry(
        pool: &SqlitePool,
        tenant_id: i64,
        dto: &CreateJournalEntryRequest,
        created_by: Option<i64>,
    ) -> Result<JournalEntry, AppError> {
        if dto.details.is_empty() {
            return Err(AppError::Validation("Journal entry needs at least one detail line".into()));
        }
        // Accumulate in Decimal to avoid float drift when comparing debit vs credit.
        // Values are stored as f64 (SQLite REAL), but the balance gate must not
        // reject an entry because 0.1 + 0.2 != 0.3 in binary float. Amounts are
        // rounded to 4 decimal places (well beyond currency precision) before
        // the balance comparison.
        let mut debit_total = rust_decimal::Decimal::ZERO;
        let mut credit_total = rust_decimal::Decimal::ZERO;
        for d in &dto.details {
            let debit = rust_decimal::Decimal::from_f64_retain(d.debit.unwrap_or_default())
                .ok_or_else(|| AppError::Validation("Invalid debit amount".into()))?;
            let credit = rust_decimal::Decimal::from_f64_retain(d.credit.unwrap_or_default())
                .ok_or_else(|| AppError::Validation("Invalid credit amount".into()))?;
            if debit.is_zero() && credit.is_zero() {
                return Err(AppError::Validation("Each detail line needs debit or credit".into()));
            }
            if !debit.is_zero() && !credit.is_zero() {
                return Err(AppError::Validation("A detail line cannot have both debit and credit".into()));
            }
            // Account must exist in this tenant.
            AccountRepo::find_by_id(pool, tenant_id, d.account_id)
                .await?
                .ok_or_else(|| AppError::Validation(format!("Unknown account id: {}", d.account_id)))?;
            debit_total += debit;
            credit_total += credit;
        }
        if debit_total.round_dp(4) != credit_total.round_dp(4) {
            return Err(AppError::Validation(format!(
                "Journal entry is unbalanced: debit {} != credit {}",
                debit_total, credit_total
            )));
        }

        let entry_no = format!("JE-{}-{}", chrono::Utc::now().format("%Y%m%d"), next_sequence(pool, "journal_entries").await?);
        let mut tx = pool.begin().await.map_err(AppError::from)?;
        let entry = JournalEntryRepo::create(&mut tx, tenant_id, &entry_no, dto.entry_date, dto.description.as_deref(), created_by)
            .await
            .map_err(AppError::from)?;
        for d in &dto.details {
            JournalEntryRepo::insert_detail(
                &mut tx,
                entry.id,
                d.account_id,
                d.debit.unwrap_or_default(),
                d.credit.unwrap_or_default(),
                d.description.as_deref(),
            )
            .await
            .map_err(AppError::from)?;
        }
        JournalEntryRepo::post(&mut tx, entry.id).await.map_err(AppError::from)?;
        tx.commit().await.map_err(AppError::from)?;
        // Re-read: the in-memory entry predates the post() status update.
        let entry = JournalEntryRepo::find_by_id(pool, tenant_id, entry.id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Journal entry not found: {}", entry.id)))?;
        Ok(entry)
    }

    pub async fn list_journal_entries(
        pool: &SqlitePool,
        tenant_id: i64,
        from: Option<chrono::NaiveDate>,
        to: Option<chrono::NaiveDate>,
    ) -> Result<Vec<JournalEntry>, AppError> {
        JournalEntryRepo::list(pool, tenant_id, from, to)
            .await
            .map_err(AppError::from)
    }

    pub async fn get_journal_entry(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
    ) -> Result<(JournalEntry, Vec<JournalEntryDetail>), AppError> {
        let entry = JournalEntryRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Journal entry not found: {}", id)))?;
        let details = JournalEntryRepo::details_for_entry(pool, id).await.map_err(AppError::from)?;
        Ok((entry, details))
    }

    pub async fn trial_balance(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<TrialBalanceRow>, AppError> {
        JournalEntryRepo::trial_balance(pool, tenant_id)
            .await
            .map_err(AppError::from)
    }

    // -----------------------------------------------------------------------
    // Invoices
    // -----------------------------------------------------------------------

    pub async fn create_invoice(
        pool: &SqlitePool,
        tenant_id: i64,
        dto: &CreateInvoiceRequest,
    ) -> Result<FinanceInvoice, AppError> {
        if !matches!(dto.invoice_type.as_str(), "sales" | "purchase") {
            return Err(AppError::Validation(format!("Invalid invoice type: {}", dto.invoice_type)));
        }
        // Compute totals from items (amount sum) + tax.
        let mut amount = 0.0f64;
        for item in &dto.items {
            let qty = item.quantity.unwrap_or(1.0);
            let unit = item.unit_price.unwrap_or_default();
            amount += qty * unit;
        }
        if let Some(amt) = dto.amount {
            amount = amt;
        }
        let tax = dto.tax_amount.unwrap_or_default();
        let total = amount + tax;

        let invoice_no = format!("INV-{}-{}", chrono::Utc::now().format("%Y%m%d"), next_sequence(pool, "finance_invoices").await?);
        let invoice = InvoiceRepo::create(
            pool, tenant_id, &invoice_no, &dto.invoice_type, dto.party_id, dto.order_id,
            amount, tax, total, dto.due_date,
        )
        .await
        .map_err(AppError::from)?;
        for item in &dto.items {
            let qty = item.quantity.unwrap_or(1.0);
            let unit = item.unit_price.unwrap_or_default();
            InvoiceRepo::insert_item(pool, invoice.id, item.description.as_deref(), qty, unit, qty * unit)
                .await
                .map_err(AppError::from)?;
        }
        Ok(invoice)
    }

    pub async fn list_invoices(
        pool: &SqlitePool,
        tenant_id: i64,
        invoice_type: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<FinanceInvoice>, AppError> {
        InvoiceRepo::list(pool, tenant_id, invoice_type, status)
            .await
            .map_err(AppError::from)
    }

    pub async fn get_invoice(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
    ) -> Result<(FinanceInvoice, Vec<FinanceInvoiceItem>), AppError> {
        let invoice = InvoiceRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Invoice not found: {}", id)))?;
        let items = InvoiceRepo::items_for_invoice(pool, id).await.map_err(AppError::from)?;
        Ok((invoice, items))
    }

    /// Confirm a draft invoice (draft → confirmed).
    pub async fn confirm_invoice(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<FinanceInvoice, AppError> {
        let invoice = InvoiceRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Invoice not found: {}", id)))?;
        if invoice.status != "draft" {
            return Err(AppError::Validation(format!("Cannot confirm invoice in status '{}'", invoice.status)));
        }
        InvoiceRepo::update_status(pool, tenant_id, id, "confirmed")
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Invoice not found: {}", id)))
    }

    /// Void an invoice (any non-paid status → void).
    pub async fn void_invoice(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<FinanceInvoice, AppError> {
        let invoice = InvoiceRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Invoice not found: {}", id)))?;
        if invoice.status == "paid" {
            return Err(AppError::Validation("Cannot void a paid invoice".into()));
        }
        InvoiceRepo::update_status(pool, tenant_id, id, "void")
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Invoice not found: {}", id)))
    }

    // -----------------------------------------------------------------------
    // Payments
    // -----------------------------------------------------------------------

    /// Register a payment; if tied to an invoice and the invoice is fully
    /// paid, mark it paid.
    pub async fn create_payment(
        pool: &SqlitePool,
        tenant_id: i64,
        dto: &CreatePaymentRequest,
        created_by: Option<i64>,
    ) -> Result<FinancePayment, AppError> {
        if !matches!(dto.direction.as_str(), "in" | "out") {
            return Err(AppError::Validation(format!("Invalid payment direction: {}", dto.direction)));
        }
        let payment_no = format!("PAY-{}-{}", chrono::Utc::now().format("%Y%m%d"), next_sequence(pool, "finance_payments").await?);
        let method = dto.method.clone().unwrap_or_else(|| "bank_transfer".into());
        let payment = PaymentRepo::create(
            pool, tenant_id, &payment_no, dto.invoice_id, &dto.direction, dto.amount,
            &method, dto.reference.as_deref(), created_by,
        )
        .await
        .map_err(AppError::from)?;

        // Auto-settle: if the invoice's total payments now cover total_amount → paid.
        if let Some(invoice_id) = dto.invoice_id {
            let invoice = InvoiceRepo::find_by_id(pool, tenant_id, invoice_id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Invoice not found: {}", invoice_id)))?;
            let paid: f64 = sqlx::query_scalar(
                "SELECT CAST(COALESCE(SUM(amount), 0.0) AS REAL) FROM finance_payments \
                 WHERE invoice_id = ? AND direction = ?",
            )
            .bind(invoice_id)
            .bind(if invoice.invoice_type == "sales" { "in" } else { "out" })
            .fetch_one(pool)
            .await
            .map_err(AppError::from)?;
            if paid >= invoice.total_amount {
                InvoiceRepo::update_status(pool, tenant_id, invoice_id, "paid")
                    .await
                    .map_err(AppError::from)?;
            }
        }
        Ok(payment)
    }

    pub async fn list_payments(
        pool: &SqlitePool,
        tenant_id: i64,
        invoice_id: Option<i64>,
    ) -> Result<Vec<FinancePayment>, AppError> {
        PaymentRepo::list(pool, tenant_id, invoice_id)
            .await
            .map_err(AppError::from)
    }
}

/// Small per-table sequence helper for human-friendly document numbers.
async fn next_sequence(pool: &SqlitePool, table: &str) -> Result<i64, AppError> {
    let n: i64 = sqlx::query_scalar(&format!(
        "SELECT COALESCE(MAX(id), 0) + 1 FROM {}",
        table
    ))
    .fetch_one(pool)
    .await
    .map_err(AppError::from)?;
    Ok(n)
}
