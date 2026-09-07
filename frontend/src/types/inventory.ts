// 库存域类型 — 与后端 inventory_repo / check_repo Row 严格对齐。数量一律 string。

export interface StockRow {
  readonly item_id: number
  readonly item_name: string | null
  readonly sku: string | null
  readonly location_id: number | null
  readonly location_name: string | null
  readonly warehouse_id: number | null
  readonly warehouse_name: string | null
  readonly quantity: string
}

export interface InventoryLog {
  readonly id: number
  readonly item_id: number
  readonly item_name: string | null
  readonly location_id: number | null
  readonly location_name: string | null
  readonly change_type: "inbound" | "outbound" | "check_adjust" | string
  readonly quantity: string
  readonly ref_type?: string | null
  readonly ref_id?: number | null
  readonly created_at: string
}

export interface InboundOrder {
  readonly id: number
  readonly record_no: string
  readonly inbound_type: string
  readonly order_id: number | null
  readonly supplier_id: number | null
  readonly status: "draft" | "posted" | "voided" | string
  readonly notes: string | null
  readonly created_by: number | null
  readonly created_at: string
  readonly updated_at: string
}

export interface InboundItem {
  readonly id: number
  readonly inbound_id?: number
  readonly record_id?: number
  readonly item_id: number
  readonly item_name?: string | null
  readonly location_id: number | null
  readonly location_name?: string | null
  readonly quantity: string
  readonly created_at?: string
}

export interface InboundItemPayload {
  item_id: number
  location_id: number
  quantity: string
  notes?: string | null
}

export interface InboundDetail {
  readonly order: InboundOrder
  readonly items: readonly InboundItem[]
}

export interface OutboundOrder {
  readonly id: number
  readonly record_no: string
  readonly outbound_type: string
  readonly order_id: number | null
  readonly customer_id: number | null
  readonly status: "draft" | "posted" | "voided" | string
  readonly notes: string | null
  readonly created_by: number | null
  readonly created_at: string
  readonly updated_at: string
}

export interface OutboundItem {
  readonly id: number
  readonly outbound_id?: number
  readonly record_id?: number
  readonly item_id: number
  readonly item_name?: string | null
  readonly location_id: number | null
  readonly location_name?: string | null
  readonly quantity: string
  readonly created_at?: string
}

export interface OutboundItemPayload {
  item_id: number
  location_id: number
  quantity: string
  notes?: string | null
}

export interface OutboundDetail {
  readonly order: OutboundOrder
  readonly items: readonly OutboundItem[]
}

export interface CheckSession {
  readonly id: number
  readonly session_no: string
  readonly location_id: number | null
  readonly location_name: string | null
  readonly scope: string
  readonly status: "draft" | "counted" | "posted" | string
  readonly created_by: number | null
  readonly created_at: string
  readonly updated_at: string
}

export interface CheckDetailRow {
  readonly id: number
  readonly session_id: number
  readonly item_id: number
  readonly item_name: string | null
  readonly sku: string | null
  readonly location_id: number | null
  readonly system_qty: string
  readonly actual_qty: string | null
  readonly diff_qty: string | null
}

export interface CheckDetail {
  readonly session: CheckSession
  readonly details: readonly CheckDetailRow[]
}

/** ATP 查询结果（/inventory/available） */
export interface AvailableQty {
  readonly item_id: number
  readonly location_id?: number | null
  readonly available_qty: string
}
