// 报表域类型 — 与后端 reports_repo Row 对齐（金额/数量一律 string）。

export interface InventorySummaryRow {
  readonly item_id: number
  readonly sku: string
  readonly name: string
  readonly category: string | null
  readonly total_qty: string
  readonly location_count: number
}

export interface InboundOutboundRow {
  readonly log_id: number
  readonly change_type: string
  readonly item_id: number
  readonly sku: string
  readonly name: string
  readonly quantity: string
  readonly location_id: number
  readonly ref_type: string | null
  readonly ref_id: number | null
  readonly created_at: string
}

export interface SalesTrendRow {
  readonly month: string
  readonly order_count: number
  readonly total_amount: string
}

export interface FinanceSummaryRow {
  readonly account_id: number
  readonly account_code: string
  readonly account_name: string
  readonly account_type: string
  readonly total_debit: string
  readonly total_credit: string
}