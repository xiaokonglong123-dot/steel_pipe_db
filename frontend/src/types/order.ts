import type { Id } from "./common"

// 订单域类型 — 采购/销售（与后端 Row + T1.4/T1.5 投影对齐）
// 金额/数量一律 string（后端 Decimal JSON 序列化）。

/** 采购订单状态机 */
export type PurchaseStatus =
  | "draft"
  | "submitted"
  | "approved"
  | "rejected"
  | "cancelled"
  | "partially_received"
  | "received"

/** 销售订单状态机 */
export type SalesStatus =
  | "draft"
  | "submitted"
  | "approved"
  | "rejected"
  | "cancelled"
  | "shipped"

/** 后端 allowed_actions 返回的动作串（上下公开信道，用 string 兜底） */
export type OrderAction =
  | "edit"
  | "delete"
  | "submit"
  | "approve"
  | "reject"
  | "cancel"
  | "receive"
  | "ship"

export interface PurchaseOrderItem {
  readonly id: Id
  readonly order_id: Id
  readonly item_id: Id
  readonly item_name: string
  readonly quantity: string
  readonly received_qty: string
  readonly unit_price: string | null
  readonly total_price: string | null
  readonly notes: string | null
  readonly created_at?: string
}

export interface PurchaseOrder {
  readonly id: Id
  readonly order_no: string
  readonly supplier_id: Id
  readonly supplier_name: string
  readonly order_date: string
  readonly status: PurchaseStatus
  readonly doc_status: number
  readonly total_amount: string
  readonly currency: string
  readonly notes: string | null
  readonly created_by?: number | null
  readonly created_at?: string
  readonly updated_at?: string
}

/** 采购订单详情（handler 返回 { order, items, allowed_actions }） */
export interface PurchaseOrderDetail {
  readonly order: PurchaseOrder
  readonly items: readonly PurchaseOrderItem[]
  readonly allowed_actions: readonly OrderAction[]
}

export interface PurchaseItemLine {
  item_id: number
  quantity: string
  unit_price?: string
  notes?: string
}

export interface PurchaseOrderPayload {
  supplier_id: number
  order_date: string
  currency?: string
  notes?: string
  items: PurchaseItemLine[]
}

// —— 销售 ——

export interface SalesOrderItem {
  readonly id: Id
  readonly order_id: Id
  readonly item_id: Id
  readonly item_name: string
  readonly quantity: string
  readonly shipped_qty: string
  readonly unit_price: string | null
  readonly total_price: string | null
  readonly notes: string | null
  readonly created_at?: string
}

export interface SalesOrder {
  readonly id: Id
  readonly order_no: string
  readonly customer_id: Id
  readonly customer_name: string
  readonly order_date: string
  readonly status: SalesStatus
  readonly doc_status: number
  readonly total_amount: string
  readonly currency: string
  readonly notes: string | null
  readonly created_by?: number | null
  readonly created_at?: string
  readonly updated_at?: string
}

export interface SalesOrderDetail {
  readonly order: SalesOrder
  readonly items: readonly SalesOrderItem[]
  readonly allowed_actions: readonly OrderAction[]
}

export interface SalesItemLine {
  item_id: number
  quantity: string
  unit_price?: string
  notes?: string
}

export interface SalesOrderPayload {
  customer_id: number
  order_date?: string
  currency?: string
  notes?: string
  items: SalesItemLine[]
}
