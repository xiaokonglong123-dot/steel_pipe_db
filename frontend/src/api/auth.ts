import { del, get, page, post, put } from "./client"
import type { Page } from "@/types/common"
import type {
  CreateUserPayload,
  OperationLog,
  Role,
  UpdateUserPayload,
  User,
} from "@/types/auth"

// /users 与 /operation-logs 分页；/roles 返回纯数组。
export async function listUsers(pageNo = 1, pageSize = 20): Promise<Page<User>> {
  return page<User>(`/users?page=${pageNo}&page_size=${pageSize}`)
}
export async function createUser(payload: CreateUserPayload): Promise<User> {
  return post<User>("/users", payload)
}
export async function updateUser(id: number, payload: UpdateUserPayload): Promise<void> {
  await put<void>(`/users/${id}`, payload)
}
export async function deleteUser(id: number): Promise<void> {
  return del(`/users/${id}`)
}

export async function listRoles(): Promise<readonly Role[]> {
  return get<readonly Role[]>("/roles")
}

export async function listOperationLogs(pageNo = 1, pageSize = 20): Promise<Page<OperationLog>> {
  return page<OperationLog>(`/operation-logs?page=${pageNo}&page_size=${pageSize}`)
}