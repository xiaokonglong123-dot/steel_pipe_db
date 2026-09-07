import type { Id } from "./common"

export interface Account {
  readonly id: Id
  readonly code: string
  readonly name: string
  readonly parent_id: number | null
  readonly parent_name: string | null
  readonly account_type: "asset" | "liability" | "equity" | "income" | "expense"
  readonly is_active: number
  readonly created_at: string
}

export interface AccountPayload {
  code: string
  name: string
  parent_id?: number | null
  account_type: Account["account_type"]
  is_active?: number
}

export interface JournalLine {
  readonly id: number
  readonly entry_id: number
  readonly account_id: number
  readonly debit: string
  readonly credit: string
  readonly description: string | null
  readonly created_at: string
}

export interface JournalEntry {
  readonly id: Id
  readonly entry_no: string
  readonly entry_date: string
  readonly description: string | null
  readonly status: "draft" | "posted" | "voided"
  readonly ref_type: string | null
  readonly ref_id: number | null
  readonly created_by: number | null
  readonly created_at: string
  readonly updated_at: string
}

export interface JournalLineInput {
  account_id: number
  debit: string
  credit: string
  description?: string | null
}

export interface JournalEntryPayload {
  entry_date: string
  description?: string | null
  ref_type?: string | null
  ref_id?: number | null
  lines: readonly JournalLineInput[]
}

export interface Invoice {
  readonly id: Id
  readonly invoice_no: string
  readonly invoice_date: string
  readonly party_type: "supplier" | "customer"
  readonly party_id: number
  readonly party_name?: string | null
  readonly amount: string
  readonly ref_type: string | null
  readonly ref_id: number | null
  readonly status: string
  readonly created_at: string
  readonly updated_at: string
}

export interface Payment {
  readonly id: Id
  readonly payment_no: string
  readonly payment_date: string
  readonly supplier_id: number | null
  readonly supplier_name?: string | null
  readonly amount: string
  readonly invoice_id: number | null
  readonly method: string | null
  readonly notes: string | null
  readonly created_by: number | null
  readonly created_at: string
}

export interface InvoicePayload {
  invoice_no: string
  invoice_date: string
  party_type: "supplier" | "customer"
  party_id: number
  amount: string
  ref_type?: string | null
  ref_id?: number | null
}

export interface PaymentPayload {
  payment_no: string
  payment_date: string
  supplier_id?: number | null
  amount: string
  invoice_id?: number | null
  method?: string | null
  notes?: string | null
}

export interface TrialBalanceRow {
  readonly account_id: number
  readonly account_code: string
  readonly account_name: string
  readonly total_debit: string
  readonly total_credit: string
  readonly balance: string
}