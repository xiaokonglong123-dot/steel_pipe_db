import type { Id } from "./common"

/** 登录后注入前端的当前用户（/auth/login 与 /auth/me 的 user 载荷）。 */
export interface AuthUser {
  readonly id: Id
  readonly username: string
  readonly display_name: string
  readonly permissions: readonly string[]
}

/** 管理员用户列表项（/users 响应）。 */
export interface User {
  readonly id: Id
  readonly username: string
  readonly display_name: string
  readonly email: string | null
  readonly phone: string | null
  readonly is_active: boolean
  readonly created_at: string
}

export interface Role {
  readonly id: Id
  readonly name: string
  readonly description: string | null
  readonly is_system: boolean
}

export interface OperationLog {
  readonly id: Id
  readonly user_id: number | null
  readonly action: string
  readonly target_type: string | null
  readonly target_id: number | null
  readonly ip_address: string | null
  readonly created_at: string
}

export interface CreateUserPayload {
  username: string
  password: string
  display_name: string
  role_ids: readonly number[]
}

export interface UpdateUserPayload {
  display_name?: string
  email?: string | null
  phone?: string | null
  is_active?: boolean
  role_ids?: readonly number[]
  password?: string
}