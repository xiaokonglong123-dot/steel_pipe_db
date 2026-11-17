import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from 'react'
import { Link } from 'react-router-dom'

import { ApiError } from '../api/client'
import { IconAlert, IconTick, IconClose, IconInbox } from './icons'

/* ------------------------------------------------------------------ */
/* 徽章与状态                                                          */
/* ------------------------------------------------------------------ */

export function Badge({
  tone = 'none',
  children,
}: {
  tone?: 'ok' | 'warn' | 'danger' | 'none' | 'info' | 'violet'
  children: ReactNode
}) {
  return <span className={`badge is-${tone}`}>{children}</span>
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
  ok: '一致',
  moved: '位置异常',
  surplus: '盘盈',
}

/** 状态三重编码：字形 + 中文 + 颜色（颜色不单独承载语义）。 */
export function StatusBadge({ value }: { value: string }) {
  const label = STATUS_LABEL[value] ?? value
  return <span className={`mark is-${value}`}>{label}</span>
}

export function ROLE_LABEL(role: string): string {
  return role === 'admin' ? '管理员' : role === 'operator' ? '操作员' : '查看者'
}

/* ------------------------------------------------------------------ */
/* 错误与提示                                                          */
/* ------------------------------------------------------------------ */

export function Problem({ error }: { error: unknown }) {
  if (!error) return null
  const message =
    error instanceof ApiError
      ? `${error.title}${error.detail ? `：${error.detail}` : ''}`
      : error instanceof Error
        ? error.message
        : String(error)
  return (
    <div className="problem" role="alert">
      <IconAlert size={16} />
      <span>{message}</span>
    </div>
  )
}

export function Notice({ children }: { children: ReactNode }) {
  return (
    <div className="notice" role="status">
      <IconTick size={16} />
      <span>{children}</span>
    </div>
  )
}

/* ------------------------------------------------------------------ */
/* 基础控件                                                            */
/* ------------------------------------------------------------------ */

export function Field({
  label,
  children,
  hint,
}: {
  label: string
  children: ReactNode
  hint?: ReactNode
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
  return (
    <div className="loader" role="status">
      <span className="spinner" />
      <span>{label}</span>
    </div>
  )
}

export function Empty({
  children,
  hint,
  icon,
}: {
  children: ReactNode
  hint?: ReactNode
  icon?: ReactNode
}) {
  return (
    <div className="empty">
      {icon ?? <IconInbox size={30} />}
      <strong>{children}</strong>
      {hint ? <span>{hint}</span> : null}
    </div>
  )
}

export function PageHead({
  title,
  sub,
  actions,
}: {
  title: ReactNode
  sub?: ReactNode
  actions?: ReactNode
}) {
  return (
    <header className="page-head">
      <div>
        <h1>{title}</h1>
        {sub ? <div className="sub">{sub}</div> : null}
      </div>
      {actions ? <div className="page-actions">{actions}</div> : null}
    </header>
  )
}

export function SectionTitle({ children, count }: { children: ReactNode; count?: ReactNode }) {
  return (
    <div className="section-title">
      <span>{children}</span>
      {count !== undefined ? <span className="count">{count}</span> : null}
      <span className="line" />
    </div>
  )
}

export function NavLinkPill({ to, children }: { to: string; children: ReactNode }) {
  return (
    <Link className="btn btn-sm" to={to}>
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
      <span>
        共 <b className="num">{total}</b> 条 · 第 <b className="num">{page}</b> / <b className="num">{pages}</b> 页
      </span>
      <span className="spacer" />
      <button className="btn btn-sm" disabled={page <= 1} onClick={() => onChange(page - 1)}>
        上一页
      </button>
      <button className="btn btn-sm" disabled={page >= pages} onClick={() => onChange(page + 1)}>
        下一页
      </button>
    </div>
  )
}

/* ------------------------------------------------------------------ */
/* 对话框                                                              */
/* ------------------------------------------------------------------ */

function useEscape(active: boolean, onClose: () => void) {
  useEffect(() => {
    if (!active) return
    const handle = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', handle)
    return () => window.removeEventListener('keydown', handle)
  }, [active, onClose])
}

export function ConfirmDialog({
  open,
  title,
  body,
  confirmLabel = '确认',
  tone = 'accent',
  pending = false,
  onConfirm,
  onCancel,
}: {
  open: boolean
  title: string
  body?: ReactNode
  confirmLabel?: string
  tone?: 'accent' | 'danger'
  pending?: boolean
  onConfirm: () => void
  onCancel: () => void
}) {
  useEscape(open, onCancel)
  if (!open) return null
  return (
    <div className="dialog-backdrop" onMouseDown={onCancel}>
      <div
        className="dialog"
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div className="dialog-body">
          <h2>{title}</h2>
          {body ? <p>{body}</p> : null}
        </div>
        <div className="dialog-actions">
          <button className="btn" type="button" onClick={onCancel} disabled={pending}>
            取消
          </button>
          <button
            className={tone === 'danger' ? 'btn btn-danger' : 'btn btn-accent'}
            type="button"
            onClick={onConfirm}
            disabled={pending}
            autoFocus
          >
            {pending ? '处理中…' : confirmLabel}
          </button>
        </div>
      </div>
    </div>
  )
}

export function PromptDialog({
  open,
  title,
  label,
  hint,
  initial = '',
  confirmLabel = '提交',
  busy = false,
  onSubmit,
  onCancel,
}: {
  open: boolean
  title: string
  label: string
  hint?: ReactNode
  initial?: string
  confirmLabel?: string
  busy?: boolean
  onSubmit: (value: string) => void
  onCancel: () => void
}) {
  const [value, setValue] = useState(initial)
  useEffect(() => {
    if (open) setValue(initial)
  }, [open, initial])
  useEscape(open, onCancel)
  if (!open) return null
  return (
    <div className="dialog-backdrop" onMouseDown={onCancel}>
      <form
        className="dialog"
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onMouseDown={(event) => event.stopPropagation()}
        onSubmit={(event) => {
          event.preventDefault()
          onSubmit(value)
        }}
      >
        <div className="dialog-body">
          <h2>{title}</h2>
          <Field label={label} hint={hint}>
            <input
              className="input"
              type="password"
              autoFocus
              value={value}
              onChange={(event) => setValue(event.target.value)}
            />
          </Field>
        </div>
        <div className="dialog-actions">
          <button className="btn" type="button" onClick={onCancel} disabled={busy}>
            取消
          </button>
          <button className="btn btn-accent" type="submit" disabled={busy || !value}>
            {busy ? '提交中…' : confirmLabel}
          </button>
        </div>
      </form>
    </div>
  )
}

/* ------------------------------------------------------------------ */
/* Toast                                                               */
/* ------------------------------------------------------------------ */

type ToastTone = 'ok' | 'danger'
type ToastItem = { id: number; tone: ToastTone; message: string }
type ToastApi = { push: (tone: ToastTone, message: string) => void }

const ToastContext = createContext<ToastApi | null>(null)

export function ToastProvider({ children }: { children: ReactNode }) {
  const [items, setItems] = useState<ToastItem[]>([])
  const seq = useRef(0)
  const timers = useRef<number[]>([])

  const push = useCallback((tone: ToastTone, message: string) => {
    const id = ++seq.current
    setItems((current) => [...current, { id, tone, message }])
    const timer = window.setTimeout(() => {
      setItems((current) => current.filter((item) => item.id !== id))
    }, 4200)
    timers.current.push(timer)
  }, [])

  useEffect(() => () => timers.current.forEach(window.clearTimeout), [])

  const api = useMemo(() => ({ push }), [push])

  return (
    <ToastContext.Provider value={api}>
      {children}
      <div className="toasts" aria-live="polite">
        {items.map((item) => (
          <div key={item.id} className={`toast is-${item.tone}`} role="status">
            <span className={`icon is-${item.tone}`}>
              {item.tone === 'ok' ? <IconTick size={16} /> : <IconAlert size={16} />}
            </span>
            <span>{item.message}</span>
            <button
              className="close"
              type="button"
              aria-label="关闭提示"
              onClick={() => setItems((current) => current.filter((row) => row.id !== item.id))}
            >
              <IconClose size={14} />
            </button>
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  )
}

export function useToast(): ToastApi {
  const value = useContext(ToastContext)
  if (!value) throw new Error('useToast 必须在 ToastProvider 内使用')
  return value
}