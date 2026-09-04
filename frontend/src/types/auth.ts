import type { Id } from "./common"

export interface User {
  readonly id: Id
  readonly username: string
  readonly display_name: string
  readonly role_id: number | null
  readonly is_active: number
  readonly permissions: readonly string[]
  readonly created_at?: string
}

export interface Role {
  readonly id: Id
  readonly name: string
  readonly description: string | null
  readonly permissions: readonly string[]
}

export interface OperationLog {
  readonly id: Id
  readonly user_id: number | null
  readonly username: string | null
  readonly action: string
  readonly detail: string | null
  readonly ip: string | null
  readonly created_at: string
}

export interface UserPayload {
  username: string
  display_name: string
  role_id: number
  password?: string
  is_active?: number
}