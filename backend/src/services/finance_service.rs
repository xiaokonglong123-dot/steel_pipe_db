use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::BTreeMap;
use std::str::FromStr;

use crate::error::{AppError, ErrorCode};
use crate::middleware::auth::AuthUser;
use crate::repos::finance_repo::{self, AccountRow, InvoiceRow, JournalEntryRow, PaymentRow};

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAccountRequest {
    pub code: String,
    pub name: String,
    pub parent_id: Option<i64>,
    pub account_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateJournalEntryRequest {
    pub entry_date: String,
    pub description: Option<String>,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub lines: Vec<JournalLineInput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JournalLineInput {
    pub account_id: i64,
    pub debit: String,
    pub credit: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateInvoiceRequest {
    pub invoice_no: String,
    pub invoice_date: String,
    pub party_type: String,
    pub party_id: i64,
    pub amount: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePaymentRequest {
    pub payment_no: String,
    pub payment_date: String,
    pub supplier_id: Option<i64>,
    pub amount: String,
    pub invoice_id: Option<i64>,
    pub method: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrialBalanceRow {
    pub account_id: i64,
    pub account_code: String,
    pub account_name: String,
    pub total_debit: String,
    pub total_credit: String,
    pub balance: String,
}

fn generate_entry_no() -> String {
    let date = Utc::now().format("%Y%m%d").to_string();
    let random = uuid::Uuid::new_v4().simple().to_string();
    format!("JE{date}-{}", &random[..4])
}

fn generate_invoice_no() -> String {
    let date = Utc::now().format("%Y%m%d").to_string();
    let random = uuid::Uuid::new_v4().simple().to_string();
    format!("INV{date}-{}", &random[..4])
}

fn generate_payment_no() -> String {
    let date = Utc::now().format("%Y%m%d").to_string();
    let random = uuid::Uuid::new_v4().simple().to_string();
    format!("PAY{date}-{}", &random[..4])
}

fn parse_amount(value: &str, message: &str) -> Result<Decimal, AppError> {
    Decimal::from_str(value).map_err(|_| AppError::validation(message))
}

pub async fn create_account(
    pool: &SqlitePool,
    dto: &CreateAccountRequest,
    _user: &AuthUser,
) -> Result<AccountRow, AppError> {
    if !matches!(
        dto.account_type.as_str(),
        "asset" | "liability" | "equity" | "income" | "expense"
    ) {
        return Err(AppError::validation("无效的会计科目类型"));
    }
    if dto.code.trim().is_empty() || dto.name.trim().is_empty() {
        return Err(AppError::validation("会计科目编码和名称不能为空"));
    }
    if let Some(parent_id) = dto.parent_id {
        if finance_repo::find_account_by_id(pool, parent_id)
            .await?
            .is_none()
        {
            return Err(AppError::new(
                ErrorCode::AccountNotFound,
                "父级会计科目未找到",
            ));
        }
    }
    finance_repo::insert_account(pool, dto).await
}

pub async fn list_accounts(
    pool: &SqlitePool,
    account_type: Option<&str>,
    active_only: bool,
) -> Result<Vec<AccountRow>, AppError> {
    if let Some(account_type) = account_type {
        if !matches!(
            account_type,
            "asset" | "liability" | "equity" | "income" | "expense"
        ) {
            return Err(AppError::validation("无效的会计科目类型"));
        }
    }
    finance_repo::list_accounts(pool, account_type, active_only).await
}

pub async fn create_journal_entry(
    pool: &SqlitePool,
    dto: &CreateJournalEntryRequest,
    user: &AuthUser,
) -> Result<JournalEntryRow, AppError> {
    if dto.lines.is_empty() {
        return Err(AppError::validation("日记账明细不能为空"));
    }
    let mut debit_total = Decimal::ZERO;
    let mut credit_total = Decimal::ZERO;
    let mut parsed = Vec::with_capacity(dto.lines.len());
    for line in &dto.lines {
        if finance_repo::find_account_by_id(pool, line.account_id)
            .await?
            .is_none()
        {
            return Err(AppError::new(ErrorCode::AccountNotFound, "会计科目未找到"));
        }
        let debit = parse_amount(&line.debit, "借方金额格式错误")?;
        let credit = parse_amount(&line.credit, "贷方金额格式错误")?;
        if debit.is_sign_negative() || credit.is_sign_negative() {
            return Err(AppError::validation("借贷金额不能为负数"));
        }
        if debit > Decimal::ZERO && credit > Decimal::ZERO {
            return Err(AppError::validation("同一行不能同时有借方和贷方金额"));
        }
        debit_total += debit;
        credit_total += credit;
        parsed.push((line, debit, credit));
    }
    if debit_total.round_dp(4) != credit_total.round_dp(4) {
        return Err(AppError::new(ErrorCode::UnbalancedJournal, "借贷不平衡"));
    }
    let mut tx = pool.begin().await?;
    let entry_id = finance_repo::insert_journal_entry_tx(
        &mut tx,
        &generate_entry_no(),
        &dto.entry_date,
        dto.description.as_deref(),
        dto.ref_type.as_deref(),
        dto.ref_id,
        user.id,
    )
    .await?;
    for (line, debit, credit) in parsed {
        finance_repo::insert_journal_entry_line_tx(
            &mut tx,
            entry_id,
            line.account_id,
            &debit.to_string(),
            &credit.to_string(),
            line.description.as_deref(),
        )
        .await?;
    }
    tx.commit().await?;
    finance_repo::find_journal_entry_by_id(pool, entry_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "日记账创建后读取失败"))
}

pub async fn list_journal_entries(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
) -> Result<(Vec<JournalEntryRow>, i64), AppError> {
    finance_repo::list_journal_entries(pool, page.max(1), page_size.clamp(1, 200)).await
}

pub async fn post_journal_entry(pool: &SqlitePool, id: i64) -> Result<JournalEntryRow, AppError> {
    let entry = finance_repo::find_journal_entry_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::JournalNotFound, "日记账未找到"))?;
    if entry.status != "draft" {
        return Err(AppError::new(
            ErrorCode::StatusConflict,
            "日记账当前状态不可过账",
        ));
    }
    finance_repo::update_journal_entry_status(pool, id, "posted").await?;
    finance_repo::find_journal_entry_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::JournalNotFound, "日记账未找到"))
}

pub async fn create_invoice(
    pool: &SqlitePool,
    dto: &CreateInvoiceRequest,
    _user: &AuthUser,
) -> Result<InvoiceRow, AppError> {
    if !matches!(dto.party_type.as_str(), "supplier" | "customer") {
        return Err(AppError::validation("发票往来单位类型无效"));
    }
    let amount = parse_amount(&dto.amount, "发票金额格式错误")?;
    if amount <= Decimal::ZERO {
        return Err(AppError::validation("发票金额必须大于 0"));
    }
    let request = CreateInvoiceRequest {
        invoice_no: if dto.invoice_no.trim().is_empty() {
            generate_invoice_no()
        } else {
            dto.invoice_no.clone()
        },
        ..dto.clone()
    };
    finance_repo::insert_invoice(pool, &request, &amount.to_string()).await
}

pub async fn list_invoices(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
) -> Result<(Vec<InvoiceRow>, i64), AppError> {
    finance_repo::list_invoices(pool, page.max(1), page_size.clamp(1, 200)).await
}

pub async fn create_payment(
    pool: &SqlitePool,
    dto: &CreatePaymentRequest,
    user: &AuthUser,
) -> Result<PaymentRow, AppError> {
    let amount = parse_amount(&dto.amount, "付款金额格式错误")?;
    if amount <= Decimal::ZERO {
        return Err(AppError::validation("付款金额必须大于 0"));
    }
    let request = CreatePaymentRequest {
        payment_no: if dto.payment_no.trim().is_empty() {
            generate_payment_no()
        } else {
            dto.payment_no.clone()
        },
        ..dto.clone()
    };
    let mut tx = pool.begin().await?;
    if let Some(invoice_id) = request.invoice_id {
        let invoice = finance_repo::find_invoice_by_id_tx(&mut tx, invoice_id)
            .await?
            .ok_or_else(|| AppError::new(ErrorCode::InvoiceNotFound, "发票未找到"))?;
        if invoice.status == "paid" {
            return Err(AppError::new(ErrorCode::InvoiceAlreadyPaid, "发票已支付"));
        }
        let paid_total = finance_repo::paid_amount_for_invoice(&mut tx, invoice_id)
            .await?
            .into_iter()
            .try_fold(Decimal::ZERO, |sum, value| {
                Ok::<Decimal, AppError>(sum + parse_amount(&value, "付款金额数据损坏")?)
            })?;
        let invoice_amount = parse_amount(&invoice.amount, "发票金额数据损坏")?;
        let total = (paid_total + amount).round_dp(4);
        let status = if total >= invoice_amount {
            "paid"
        } else {
            "partially_paid"
        };
        finance_repo::insert_payment_tx(&mut tx, &request, &amount.to_string(), user.id).await?;
        finance_repo::update_invoice_status_tx(&mut tx, invoice_id, status).await?;
    } else {
        finance_repo::insert_payment_tx(&mut tx, &request, &amount.to_string(), user.id).await?;
    }
    tx.commit().await?;
    let payment = sqlx::query_as::<_, PaymentRow>("SELECT id, payment_no, payment_date, supplier_id, amount, invoice_id, method, notes, created_by, created_at FROM payments WHERE payment_no = ?").bind(&request.payment_no).fetch_optional(pool).await?;
    payment.ok_or_else(|| AppError::new(ErrorCode::Internal, "付款创建后读取失败"))
}

pub async fn list_payments(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
) -> Result<(Vec<PaymentRow>, i64), AppError> {
    finance_repo::list_payments(pool, page.max(1), page_size.clamp(1, 200)).await
}

pub async fn trial_balance(pool: &SqlitePool) -> Result<Vec<TrialBalanceRow>, AppError> {
    let mut totals: BTreeMap<i64, (String, String, Decimal, Decimal)> = BTreeMap::new();
    for line in finance_repo::posted_lines(pool).await? {
        let debit = parse_amount(&line.debit, "借方金额数据损坏")?;
        let credit = parse_amount(&line.credit, "贷方金额数据损坏")?;
        let entry = totals.entry(line.account_id).or_insert((
            line.account_code,
            line.account_name,
            Decimal::ZERO,
            Decimal::ZERO,
        ));
        entry.2 += debit;
        entry.3 += credit;
    }
    Ok(totals
        .into_iter()
        .map(
            |(account_id, (account_code, account_name, debit, credit))| TrialBalanceRow {
                account_id,
                account_code,
                account_name,
                total_debit: debit.round_dp(4).to_string(),
                total_credit: credit.round_dp(4).to_string(),
                balance: (debit - credit).round_dp(4).to_string(),
            },
        )
        .collect())
}
