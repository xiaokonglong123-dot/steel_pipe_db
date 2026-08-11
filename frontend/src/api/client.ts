import { useAuthStore } from "@/stores/auth"
import type { ApiEnvelope, ApiPage } from "@/types"

export class ApiError extends Error {
  readonly name = "ApiError"
  constructor(readonly status: number, message: string, readonly code?: number) {
    super(message)
  }
}

type ErrorBody = { readonly success?: false; readonly code?: number; readonly message?: string }

const baseURL = "/api/v1"

export async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const controller = new AbortController()
  const timeout = window.setTimeout(() => controller.abort(), 30_000)
  const auth = useAuthStore()
  const headers = new Headers(options.headers)
  headers.set("Content-Type", "application/json")
  if (auth.auth_token) headers.set("Authorization", `Bearer ${auth.auth_token}`)
  try {
    const response = await fetch(`${baseURL}${path}`, { ...options, headers, credentials: "include", signal: controller.signal })
    const payload: unknown = await response.json()
    if (!response.ok) {
      const body = payload as ErrorBody
      if (response.status === 401) {
        auth.clearAuth()
        window.location.assign("/login")
      }
      throw new ApiError(response.status, body.message ?? "请求失败", body.code)
    }
    const envelope = payload as ApiEnvelope<T>
    return envelope.data
  } finally {
    window.clearTimeout(timeout)
  }
}

export function get<T>(path: string): Promise<T> { return request<T>(path) }
export function post<T>(path: string, body?: unknown): Promise<T> { return request<T>(path, { method: "POST", body: JSON.stringify(body ?? {}) }) }
export function put<T>(path: string, body: unknown): Promise<T> { return request<T>(path, { method: "PUT", body: JSON.stringify(body) }) }
export function del(path: string): Promise<void> { return request<void>(path, { method: "DELETE" }) }
export type Page<T> = ApiPage<T>
