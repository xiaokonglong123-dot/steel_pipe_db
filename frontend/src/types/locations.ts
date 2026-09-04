import type { Id } from "./common"

export interface Warehouse {
  readonly id: Id
  readonly code: string
  readonly name: string
  readonly address: string | null
  readonly created_at: string
  readonly updated_at: string
}

export interface Location {
  readonly id: Id
  readonly warehouse_id: number | null
  readonly warehouse_name: string | null
  readonly code: string
  readonly name: string
  readonly created_at: string
  readonly updated_at: string
}

export interface WarehousePayload {
  code: string
  name: string
  address?: string | null
}

export interface LocationPayload {
  warehouse_id?: number | null
  code: string
  name: string
}