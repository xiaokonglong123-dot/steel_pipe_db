import type { components } from './schema'

/** OpenAPI 生成的 schema；页面类型统一从这里取，避免手写 DTO 漂移。 */
export type Schemas = components['schemas']

export class ApiError extends Error {
  constructor(
    readonly status: number,
    readonly title: string,
    readonly detail: string,
  ) {
    super(detail || title)
    this.name = 'ApiError'
  }
}

function readCookie(name: string): string | null {
  const prefix = `${name}=`
  for (const part of document.cookie.split(';')) {
    const trimmed = part.trim()
    if (trimmed.startsWith(prefix)) return decodeURIComponent(trimmed.slice(prefix.length))
  }
  return null
}

async function problemOf(response: Response): Promise<ApiError> {
  const body: unknown = await response.json().catch(() => null)
  if (body && typeof body === 'object' && 'title' in body) {
    const problem = body as { title?: string; detail?: string }
    return new ApiError(response.status, problem.title ?? '请求失败', problem.detail ?? '')
  }
  return new ApiError(response.status, '请求失败', `HTTP ${response.status}`)
}

let refreshing: Promise<boolean> | null = null

/** 401 时用 refresh cookie 静默刷新一次（A13）。 */
async function refreshOnce(): Promise<boolean> {
  if (!refreshing) {
    refreshing = (async () => {
      const csrf = readCookie('csrf_token')
      if (!csrf) return false
      const response = await fetch('/api/auth/refresh', {
        method: 'POST',
        headers: { 'x-csrf-token': csrf, 'content-type': 'application/json' },
        credentials: 'same-origin',
        body: '{}',
      })
      return response.ok
    })().finally(() => {
      refreshing = null
    })
  }
  return refreshing
}

type RequestOptions = {
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE'
  body?: unknown
  raw?: string
  contentType?: string
  skipRefresh?: boolean
}

async function send(path: string, options: RequestOptions): Promise<Response> {
  const headers: Record<string, string> = { accept: 'application/json' }
  const method = options.method ?? 'GET'

  if (options.raw !== undefined) {
    headers['content-type'] = options.contentType ?? 'text/csv'
  } else if (options.body !== undefined) {
    headers['content-type'] = 'application/json'
  }
  if (method !== 'GET') {
    const csrf = readCookie('csrf_token')
    if (csrf) headers['x-csrf-token'] = csrf
  }

  return fetch(path, {
    method,
    headers,
    credentials: 'same-origin',
    body:
      options.raw !== undefined
        ? options.raw
        : options.body !== undefined
          ? JSON.stringify(options.body)
          : undefined,
  })
}

export async function api<T>(path: string, options: RequestOptions = {}): Promise<T> {
  let response = await send(path, options)

  if (response.status === 401 && !options.skipRefresh && !path.startsWith('/api/auth/login')) {
    if (await refreshOnce()) {
      response = await send(path, options)
    }
  }

  if (!response.ok) throw await problemOf(response)
  if (response.status === 204) return undefined as T
  return (await response.json()) as T
}

export const csrf = {
  /** 首屏领取 CSRF token（登录前也要）。 */
  async ensure(): Promise<string> {
    const existing = readCookie('csrf_token')
    if (existing) return existing
    const response = await fetch('/api/auth/csrf', { credentials: 'same-origin' })
    const body = (await response.json()) as { csrf_token: string }
    return body.csrf_token
  },
}

export type { components }
