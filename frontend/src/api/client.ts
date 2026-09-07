import { useAuthStore } from "@/stores/auth"
import type { ApiEnvelope, Page, PageMeta } from "@/types/common"

export class ApiError extends Error {
  readonly name = "ApiError"
  constructor(readonly status: number, message: string, readonly code?: number) {
    super(message)
  }
}

type ErrorBody = { readonly success?: false; readonly code?: number; readonly message?: string }

const baseURL = "/api"

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

/**
 * 分页 GET — 把后端 data.items + meta 合并成 Page<T>，供列表页统一使用。
 *
 * 后端结构：{ success, request_id, data: { items }, meta: { total, page, page_size, total_pages } }
 * 本函数返回 { items, total, page, page_size, total_pages }。
 */
export async function page<T>(
  path: string,
  options: RequestInit = {},
): Promise<Page<T>> {
  const envelope = await rawFetch(path, options)
  const data = (envelope.data ?? {}) as { readonly items?: readonly T[] }
  const meta = (envelope.meta ?? {}) as Partial<PageMeta>
  return {
    items: data.items ?? [],
    total: meta.total ?? (data.items as readonly unknown[] | undefined)?.length ?? 0,
    page: meta.page ?? 1,
    page_size: meta.page_size ?? (data.items ?? [])?.length ?? 0,
    total_pages: meta.total_pages ?? 1,
  }
}

/** 底层 fetch：返回完整 envelope（含 data + meta），不带 auth 自动跳登录已内置。 */
async function rawFetch<T>(path: string, options: RequestInit = {}): Promise<
  ApiEnvelope<T> & { readonly meta?: PageMeta }
> {
  const controller = new AbortController()
  const timeout = window.setTimeout(() => controller.abort(), 30_000)
  const auth = useAuthStore()
  const headers = new Headers(options.headers)
  headers.set("Content-Type", "application/json")
  if (auth.auth_token) headers.set("Authorization", `Bearer ${auth.auth_token}`)
  try {
    const response = await fetch(`${baseURL}${path}`, {
      ...options,
      headers,
      credentials: "include",
      signal: controller.signal,
    })
    const payload: unknown = await response.json()
    if (!response.ok) {
      const body = payload as ErrorBody
      if (response.status === 401) {
        auth.clearAuth()
        window.location.assign("/login")
      }
      throw new ApiError(response.status, body.message ?? "请求失败", body.code)
    }
    return payload as ApiEnvelope<T> & { readonly meta?: PageMeta }
  } finally {
    window.clearTimeout(timeout)
  }
}
