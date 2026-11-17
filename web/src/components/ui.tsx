import type { ReactNode } from 'react'
import { Link } from 'react-router-dom'

import { ApiError } from '../api/client'

export function Badge({
  tone = 'none',
  children,
}: {
  tone?: 'ok' | 'warn' | 'danger' | 'none' | 'info' | 'violet'
  children: ReactNode
}) {
  return <span className={`badge is-${tone}`}>{children}</span>
}

const STATUS_TONE: Record<string, 'ok' | 'warn' | 'danger' | 'none' | 'info'> = {
  in_stock: 'ok',
  in_transfer: 'warn',
  out_stock: 'none',
  missing: 'danger',
  normal: 'ok',
  problematic: 'warn',
  scrapped: 'danger',
  open: 'info',
  confirmed: 'ok',
  cancelled: 'none',
}

const STATUS_LABEL: Record<string, string> = {
  in_stock: '在库',
  in_transfer: '调拨中',
  out_stock: '已出库',
  missing: '盘亏',
  normal: '正常',
  problematic: '问题管',
  scrapped: '报废',
  open: '进行中',
  confirmed: '已确认',
  cancelled: '已取消',
}

export function StatusBadge({ value }: { value: string }) {
  return <Badge tone={STATUS_TONE[value] ?? 'none'}>{STATUS_LABEL[value] ?? value}</Badge>
}

export function ROLE_LABEL(role: string): string {
  return role === 'admin' ? '管理员' : role === 'operator' ? '操作员' : '查看者'
}

export function Problem({ error }: { error: unknown }) {
  if (!error) return null
  const message =
    error instanceof ApiError
      ? `${error.title}${error.detail ? `：${error.detail}` : ''}`
      : error instanceof Error
        ? error.message
        : String(error)
  return <div className="problem">{message}</div>
}

export function Field({
  label,
  children,
  hint,
}: {
  label: string
  children: ReactNode
  hint?: string
}) {
  return (
    <label className="field">
      <span className="field-label">{label}</span>
      {children}
      {hint ? <span className="field-hint">{hint}</span> : null}
    </label>
  )
}

export function Loader({ label = '加载中…' }: { label?: string }) {
  return <div className="muted">{label}</div>
}

export function Empty({ children }: { children: ReactNode }) {
  return <div className="empty">{children}</div>
}

export function PageHead({
  title,
  actions,
}: {
  title: string
  actions?: ReactNode
}) {
  return (
    <header className="page-head">
      <h1>{title}</h1>
      <div className="page-actions">{actions}</div>
    </header>
  )
}

export function NavLinkPill({ to, children }: { to: string; children: ReactNode }) {
  return (
    <Link className="nav-link" to={to}>
      {children}
    </Link>
  )
}

export function Pager({
  page,
  pageSize,
  total,
  onChange,
}: {
  page: number
  pageSize: number
  total: number
  onChange: (page: number) => void
}) {
  const pages = Math.max(1, Math.ceil(total / pageSize))
  return (
    <div className="pager">
      <button className="btn btn-subtle" disabled={page <= 1} onClick={() => onChange(page - 1)}>
        上一页
      </button>
      <span className="muted">
        {page} / {pages} · 共 {total} 条
      </span>
      <button
        className="btn btn-subtle"
        disabled={page >= pages}
        onClick={() => onChange(page + 1)}
      >
        下一页
      </button>
    </div>
  )
}
