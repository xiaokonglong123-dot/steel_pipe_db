import { useEffect, useState } from 'react'

type Health = {
  status: string
  version: string
  commit: string
  uptime_seconds: number
}

type DbHealth = {
  status: string
  latency_ms: number
}

type Probe<T> =
  | { state: 'loading' }
  | { state: 'ok'; data: T }
  | { state: 'error'; message: string }

async function probe<T>(path: string): Promise<Probe<T>> {
  try {
    const response = await fetch(path, { headers: { accept: 'application/json' } })
    if (!response.ok) {
      const problem: unknown = await response.json().catch(() => null)
      const detail =
        problem && typeof problem === 'object' && 'detail' in problem
          ? String((problem as { detail: unknown }).detail)
          : `HTTP ${response.status}`
      return { state: 'error', message: detail }
    }
    return { state: 'ok', data: (await response.json()) as T }
  } catch (error) {
    return { state: 'error', message: error instanceof Error ? error.message : String(error) }
  }
}

export default function App() {
  const [health, setHealth] = useState<Probe<Health>>({ state: 'loading' })
  const [db, setDb] = useState<Probe<DbHealth>>({ state: 'loading' })

  useEffect(() => {
    void probe<Health>('/api/health').then(setHealth)
    void probe<DbHealth>('/api/health/db').then(setDb)
  }, [])

  return (
    <main className="boot">
      <h1>SteelPipe</h1>
      <p className="boot-sub">M0 骨架 · 单二进制 + PostgreSQL · 前端 SPA</p>

      <dl className="boot-grid">
        <Row label="进程" probe={health} render={(h) => `v${h.version} · ${h.commit} · 已运行 ${h.uptime_seconds}s`} />
        <Row label="数据库" probe={db} render={(h) => `可达 · ${h.latency_ms}ms`} />
      </dl>

      <p className="boot-note">
        业务页面自 M6 落地。当前可访问 <code>/api/health</code>、<code>/api/health/db</code>、
        <code>/api/openapi.json</code>。
      </p>
    </main>
  )
}

function Row<T>({
  label,
  probe,
  render,
}: {
  label: string
  probe: Probe<T>
  render: (data: T) => string
}) {
  let badgeClass: string
  let text: string
  if (probe.state === 'ok') {
    badgeClass = 'is-ok'
    text = render(probe.data)
  } else if (probe.state === 'error') {
    badgeClass = 'is-danger'
    text = probe.message
  } else {
    badgeClass = 'is-none'
    text = '检测中…'
  }

  return (
    <>
      <dt>{label}</dt>
      <dd>
        <span className={`badge ${badgeClass}`}>{text}</span>
      </dd>
    </>
  )
}
