import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import { NavLink, useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { useSession } from '../session'
import {
  IconAudit,
  IconBox,
  IconCheck,
  IconGauge,
  IconImport,
  IconInbound,
  IconPin,
  IconPipe,
  IconSearch,
  IconStandard,
  IconUsers,
} from './icons'
import { ROLE_LABEL, useToast } from './ui'

type Stats = Schemas['DashboardDto']

/* ------------------------------------------------------------------ */
/* 左侧图标轨                                                          */
/* ------------------------------------------------------------------ */

type NavItem = {
  to: string
  label: string
  icon: typeof IconGauge
  write?: boolean
  admin?: boolean
  count?: (stats: Stats | undefined) => number
}

const NAV_GROUPS: { items: NavItem[] }[] = [
  {
    items: [
      { to: '/', label: '仪表盘', icon: IconGauge },
      { to: '/tubes', label: '管子', icon: IconPipe },
      { to: '/stock-in', label: '入库', icon: IconInbound, write: true },
      { to: '/import', label: '导入', icon: IconImport, write: true },
    ],
  },
  {
    items: [
      { to: '/locations', label: '库位', icon: IconPin },
      { to: '/standards', label: '规格', icon: IconStandard },
    ],
  },
  {
    items: [
      {
        to: '/checks',
        label: '盘点',
        icon: IconCheck,
        count: (stats) => stats?.tubes.missing ?? 0,
      },
    ],
  },
  {
    items: [
      { to: '/users', label: '用户', icon: IconUsers, admin: true },
      { to: '/audit', label: '审计', icon: IconAudit, admin: true },
    ],
  },
]

export function SideRail({ stats }: { stats: Stats | undefined }) {
  const { isAdmin, canWrite } = useSession()

  return (
    <nav className="rail" aria-label="主导航">
      <div className="rail-brand" title="SteelPipe">
        <IconBox size={22} />
      </div>
      {NAV_GROUPS.map((group, index) => {
        const items = group.items.filter(
          (item) => (!item.admin || isAdmin) && (!item.write || canWrite),
        )
        if (items.length === 0) return null
        return (
          <div className="rail-group" key={index}>
            {items.map((item) => {
              const count = item.count?.(stats) ?? 0
              const Icon = item.icon
              return (
                <NavLink
                  key={item.to}
                  to={item.to}
                  end={item.to === '/'}
                  className={({ isActive }) => (isActive ? 'rail-item is-active' : 'rail-item')}
                  title={item.label}
                >
                  <Icon size={20} />
                  <span>{item.label}</span>
                  {count > 0 ? <span className="rail-count">{count}</span> : null}
                </NavLink>
              )
            })}
          </div>
        )
      })}
      <div className="rail-foot">v0.1</div>
    </nav>
  )
}

/* ------------------------------------------------------------------ */
/* 密度                                                                */
/* ------------------------------------------------------------------ */

const DENSITY_KEY = 'steelpipe.density'

export type Density = 'compact' | 'comfortable' | 'roomy'

/** 阅读密度偏好：写入 <html data-density> 并持久化。 */
export function useDensity(): [Density, (next: Density) => void] {
  const [density, setDensity] = useState<Density>(() => {
    const stored = window.localStorage.getItem(DENSITY_KEY)
    return stored === 'compact' || stored === 'roomy' ? stored : 'comfortable'
  })

  useEffect(() => {
    const html = document.documentElement
    if (density === 'comfortable') delete html.dataset.density
    else html.dataset.density = density
    window.localStorage.setItem(DENSITY_KEY, density)
  }, [density])

  return [density, setDensity]
}

const DENSITY_LABELS: { value: Density; label: string; title: string }[] = [
  { value: 'compact', label: '密', title: '紧凑：36px 行高' },
  { value: 'comfortable', label: '适', title: '舒适：42px 行高（默认）' },
  { value: 'roomy', label: '宽', title: '宽松：48px 行高' },
]

function DensitySwitch() {
  const [density, setDensity] = useDensity()
  return (
    <div className="seg" role="group" aria-label="表格密度">
      {DENSITY_LABELS.map((option) => (
        <button
          key={option.value}
          type="button"
          title={option.title}
          aria-pressed={density === option.value}
          className={density === option.value ? 'is-active' : undefined}
          onClick={() => setDensity(option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
  )
}

/* ------------------------------------------------------------------ */
/* 顶栏                                                                */
/* ------------------------------------------------------------------ */

export function Topbar({ onCommand }: { onCommand: () => void }) {
  const { user, logout } = useSession()
  const { push } = useToast()

  return (
    <header className="topbar">
      <div className="brand">
        <strong>SteelPipe</strong>
        <span>钢管与筛管库存</span>
      </div>
      <button className="cmd-trigger" type="button" onClick={onCommand}>
        <IconSearch size={15} />
        <span>搜索编号 / 跳转页面</span>
        <kbd>⌘K</kbd>
      </button>
      <div className="top-right">
        <DensitySwitch />
        {user ? (
          <div className="user">
            <span className="avatar">{user.display_name.slice(0, 1).toUpperCase()}</span>
            <span>{user.display_name}</span>
            <span className="user-role">{ROLE_LABEL(user.role)}</span>
            <button
              className="btn btn-ghost btn-sm"
              type="button"
              onClick={() => {
                void logout()
                  .then(() => push('ok', '已退出登录'))
                  .catch(() => push('danger', '退出失败'))
              }}
            >
              退出
            </button>
          </div>
        ) : null}
      </div>
    </header>
  )
}

/* ------------------------------------------------------------------ */
/* KPI                                                                 */
/* ------------------------------------------------------------------ */

export function Kpi({
  label,
  value,
  unit,
  sub,
  tone,
  hero,
  meter,
}: {
  label: ReactNode
  value: number | string
  unit?: string
  sub?: ReactNode
  tone?: 'ok' | 'warn' | 'danger' | 'accent'
  hero?: boolean
  meter?: { value: number; max: number }
}) {
  return (
    <div className={`kpi${hero ? ' kpi-hero' : ''}${tone ? ` is-${tone}` : ''}`}>
      <div className="kpi-label">{label}</div>
      <div className="kpi-value">
        <span>{value}</span>
        {unit ? <span className="unit">{unit}</span> : null}
      </div>
      {meter ? <Meter value={meter.value} max={meter.max} /> : null}
      {sub ? <div className="kpi-sub">{sub}</div> : null}
    </div>
  )
}

export function Meter({
  value,
  max,
  tone,
}: {
  value: number
  max: number
  tone?: 'ok' | 'warn' | 'danger'
}) {
  const pct = max > 0 ? Math.min(100, Math.round((value / max) * 100)) : 0
  return (
    <div
      className={`meter${tone ? ` is-${tone}` : ''}`}
      role="progressbar"
      aria-valuenow={value}
      aria-valuemin={0}
      aria-valuemax={max || undefined}
      title={`${value} / ${max || '∞'}（${pct}%）`}
    >
      <i style={{ width: `${pct}%` }} />
    </div>
  )
}

/* ------------------------------------------------------------------ */
/* 命令面板                                                            */
/* ------------------------------------------------------------------ */

type Command = { id: string; label: string; hint?: string; icon: typeof IconGauge; run: () => void }

export function CommandPalette({ open, onClose }: { open: boolean; onClose: () => void }) {
  const navigate = useNavigate()
  const { isAdmin, canWrite } = useSession()
  const [query, setQuery] = useState('')
  const [active, setActive] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)

  const commands = useMemo<Command[]>(() => {
    const pages: Command[] = [
      { id: 'dash', label: '仪表盘', hint: '页面', icon: IconGauge, run: () => navigate('/') },
      { id: 'tubes', label: '管子列表', hint: '页面', icon: IconPipe, run: () => navigate('/tubes') },
      { id: 'checks', label: '盘点', hint: '页面', icon: IconCheck, run: () => navigate('/checks') },
      { id: 'locations', label: '库位', hint: '页面', icon: IconPin, run: () => navigate('/locations') },
      { id: 'standards', label: '标准规格', hint: '页面', icon: IconStandard, run: () => navigate('/standards') },
    ]
    if (canWrite) {
      pages.push({ id: 'stock-in', label: '入库', hint: '页面', icon: IconInbound, run: () => navigate('/stock-in') })
      pages.push({ id: 'import', label: '批量导入', hint: '页面', icon: IconImport, run: () => navigate('/import') })
    }
    if (isAdmin) {
      pages.push({ id: 'users', label: '用户管理', hint: '页面', icon: IconUsers, run: () => navigate('/users') })
      pages.push({ id: 'audit', label: '审计链', hint: '页面', icon: IconAudit, run: () => navigate('/audit') })
    }
    return pages
  }, [navigate, canWrite, isAdmin])

  const trimmed = query.trim()
  const filtered = useMemo(() => {
    if (!trimmed) return commands
    const lower = trimmed.toLowerCase()
    const hits = commands.filter(
      (command) => command.label.toLowerCase().includes(lower) || command.id.includes(lower),
    )
    const tube: Command = {
      id: 'tube-query',
      label: `打开管子「${trimmed}」`,
      hint: '编号',
      icon: IconPipe,
      run: () => navigate(`/tubes/${encodeURIComponent(trimmed)}`),
    }
    return [tube, ...hits]
  }, [trimmed, commands, navigate])

  useEffect(() => {
    if (!open) return
    setQuery('')
    setActive(0)
    const handle = window.setTimeout(() => inputRef.current?.focus(), 0)
    return () => window.clearTimeout(handle)
  }, [open])

  if (!open) return null

  const run = (command: Command | undefined) => {
    if (!command) return
    command.run()
    onClose()
  }

  return (
    <div className="cmd-backdrop" onMouseDown={onClose}>
      <div className="cmd-panel" role="dialog" aria-modal="true" aria-label="命令面板" onMouseDown={(event) => event.stopPropagation()}>
        <input
          ref={inputRef}
          className="cmd-input"
          placeholder="输入页面名，或直接粘贴管子编号…"
          value={query}
          onChange={(event) => {
            setQuery(event.target.value)
            setActive(0)
          }}
          onKeyDown={(event) => {
            if (event.key === 'Escape') onClose()
            if (event.key === 'ArrowDown') {
              event.preventDefault()
              setActive((current) => Math.min(current + 1, filtered.length - 1))
            }
            if (event.key === 'ArrowUp') {
              event.preventDefault()
              setActive((current) => Math.max(current - 1, 0))
            }
            if (event.key === 'Enter') {
              event.preventDefault()
              run(filtered[active])
            }
          }}
        />
        <div className="cmd-list">
          {filtered.length === 0 ? <div className="cmd-empty">没有匹配项</div> : null}
          {filtered.map((command, index) => {
            const Icon = command.icon
            return (
              <div
                key={command.id}
                className={index === active ? 'cmd-item is-active' : 'cmd-item'}
                onMouseEnter={() => setActive(index)}
                onMouseDown={() => run(command)}
              >
                <Icon size={16} />
                <span>{command.label}</span>
                {command.hint ? <span className="hint">{command.hint}</span> : null}
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}

/* ------------------------------------------------------------------ */
/* 数据查询                                                            */
/* ------------------------------------------------------------------ */

export function useStats(): Stats | undefined {
  const { user } = useSession()
  const query = useQuery({
    queryKey: ['stats'],
    queryFn: () => api<Stats>('/api/stats'),
    enabled: Boolean(user),
    refetchInterval: 30_000,
  })
  return query.data
}