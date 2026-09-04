import type { Id } from "./common"

/** 商品 SKU 主数据（active 才可交易） */
export interface Item {
  readonly id: Id
  readonly sku: string
  readonly name: string
  readonly category: string | null
  readonly unit: string | null
  readonly spec: string | null
  readonly status: "draft" | "active" | "disabled" | string
  readonly created_at: string
  readonly updated_at: string
}

export interface ItemPayload {
  sku: string
  name: string
  category?: string | null
  unit?: string | null
  spec?: string | null
  status?: string
}

export interface ImportReport {
  readonly total: number
  readonly succeeded: number
  readonly failed: number
}