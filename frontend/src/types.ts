export type ApiEnvelope<T> = { readonly success: true; readonly request_id: string; readonly data: T }
export type ApiPage<T> = { readonly items: readonly T[]; readonly total: number; readonly page: number; readonly page_size: number; readonly total_pages: number }
export type User = { readonly id: string; readonly username: string; readonly display_name: string; readonly permissions: readonly string[] }
export type Item = { readonly id: string; readonly sku: string; readonly name: string; readonly category?: string; readonly unit?: string; readonly specification?: string; readonly status?: string }
export type Party = { readonly id: string; readonly code?: string; readonly name: string; readonly contact?: string; readonly phone?: string; readonly address?: string; readonly status?: string }
export type Warehouse = { readonly id: string; readonly code?: string; readonly name: string; readonly address?: string; readonly status?: string }
export type Location = { readonly id: string; readonly warehouse_id: string; readonly code?: string; readonly name: string; readonly status?: string }
export type Stock = { readonly id?: string; readonly item_id: string; readonly location_id: string; readonly quantity: number; readonly available_quantity?: number }
export type Order = { readonly id: string; readonly order_no?: string; readonly supplier_id?: string; readonly customer_id?: string; readonly status?: string; readonly expected_date?: string; readonly total_amount?: number }
export type WorkflowInstance = { readonly id: string; readonly business_type?: string; readonly business_id?: string; readonly status?: string; readonly created_at?: string }
export type WorkflowTask = { readonly id: string; readonly title?: string; readonly status?: string; readonly assignee_id?: string; readonly created_at?: string }
export type FormRow = Record<string, string | number | undefined>

export type Account = {
  readonly id: number
  readonly code: string
  readonly name: string
  readonly parent_id?: number | null
  readonly account_type: "asset" | "liability" | "equity" | "income" | "expense"
  readonly is_active?: number | boolean
}

export type CreateAccountRequest = {
  readonly code: string
  readonly name: string
  readonly parent_id?: number | null
  readonly account_type: Account["account_type"]
}

export type JournalLine = {
  readonly id?: number
  readonly account_id: number
  readonly account_code?: string
  readonly account_name?: string
  readonly debit: string
  readonly credit: string
  readonly description?: string | null
}

export type JournalEntry = {
  readonly id: number
  readonly entry_no?: string
  readonly entry_date: string
  readonly description?: string | null
  readonly status: "draft" | "posted" | "voided"
  readonly lines?: readonly JournalLine[]
  readonly created_at?: string
}

export type JournalLineInput = {
  readonly account_id: number
  readonly debit: string
  readonly credit: string
  readonly description?: string | null
}

export type CreateJournalEntryRequest = {
  readonly entry_date: string
  readonly description?: string | null
  readonly ref_type?: string | null
  readonly ref_id?: number | null
  readonly lines: readonly JournalLineInput[]
}

export type Invoice = {
  readonly id: number
  readonly invoice_no: string
  readonly invoice_date: string
  readonly party_type: "supplier" | "customer"
  readonly party_id: number
  readonly amount: string
  readonly status: "unpaid" | "partially_paid" | "paid"
  readonly ref_type?: string | null
  readonly ref_id?: number | null
  readonly created_at?: string
}

export type CreateInvoiceRequest = {
  readonly invoice_no: string
  readonly invoice_date: string
  readonly party_type: "supplier" | "customer"
  readonly party_id: number
  readonly amount: string
  readonly ref_type?: string | null
  readonly ref_id?: number | null
}

export type Payment = {
  readonly id: number
  readonly payment_no?: string
  readonly payment_date: string
  readonly supplier_id?: number | null
  readonly amount: string
  readonly invoice_id?: number | null
  readonly method?: string | null
  readonly notes?: string | null
  readonly created_at?: string
}

export type CreatePaymentRequest = {
  readonly payment_no: string
  readonly payment_date: string
  readonly supplier_id?: number | null
  readonly amount: string
  readonly invoice_id?: number | null
  readonly method?: string | null
  readonly notes?: string | null
}

export type TrialBalanceRow = {
  readonly account_id: number
  readonly account_code: string
  readonly account_name: string
  readonly total_debit: string
  readonly total_credit: string
  readonly balance: string
}

export type FinanceSummaryRow = {
  readonly account_id: number
  readonly account_code: string
  readonly account_name: string
  readonly account_type: string
  readonly total_debit: string
  readonly total_credit: string
}

export type CheckDetail = {
  readonly id: number
  readonly session_id: number
  readonly item_id: number
  readonly system_qty: number
  readonly actual_qty?: number | null
  readonly diff_qty?: number | null
  readonly item_sku?: string
  readonly item_name?: string
}

export type CheckSession = {
  readonly id: number
  readonly location_id: number
  readonly status: "draft" | "counted" | "posted"
  readonly scope?: string
  readonly created_at?: string
  readonly details?: readonly CheckDetail[]
}

export type InventoryLog = {
  readonly id: number
  readonly item_id: number
  readonly location_id: number
  readonly change_type: "inbound" | "outbound" | "check_adjust"
  readonly quantity: number
  readonly ref_type?: string | null
  readonly ref_id?: number | null
  readonly created_at: string
}

export type AvailableQty = {
  readonly item_id: number
  readonly location_id?: number | null
  readonly available_qty: number
}

export type InventorySummaryRow = {
  readonly item_id: number
  readonly sku: string
  readonly name: string
  readonly category?: string | null
  readonly total_qty: number
  readonly location_count: number
}

export type InboundOutboundRow = {
  readonly log_id: number
  readonly change_type: "inbound" | "outbound" | "check_adjust"
  readonly item_id: number
  readonly sku: string
  readonly name: string
  readonly quantity: number
  readonly location_id: number
  readonly ref_type?: string | null
  readonly ref_id?: number | null
  readonly created_at: string
}

export type SalesTrendRow = {
  readonly month: string
  readonly order_count: number
  readonly total_amount: string
}

export interface ImportReport {
  total: number
  succeeded: number
  failed: number
  errors: string[]
}

export interface OperationLog {
  id: number
  created_at: string
  user_id: number | null
  action: string
  entity: string
  entity_id: number | null
  detail: string | null
  ip_address: string | null
}
