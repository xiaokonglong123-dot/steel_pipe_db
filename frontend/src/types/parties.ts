import type { Id } from "./common"

export interface Supplier {
  readonly id: Id
  readonly code: string
  readonly name: string
  readonly contact: string | null
  readonly phone: string | null
  readonly email: string | null
  readonly address: string | null
  readonly status: string
  readonly created_at: string
  readonly updated_at: string
}

export interface Customer {
  readonly id: Id
  readonly code: string
  readonly name: string
  readonly contact: string | null
  readonly phone: string | null
  readonly email: string | null
  readonly address: string | null
  readonly status: string
  readonly created_at: string
  readonly updated_at: string
}

export interface PartyPayload {
  code: string
  name: string
  contact?: string | null
  phone?: string | null
  email?: string | null
  address?: string | null
  status?: string
}