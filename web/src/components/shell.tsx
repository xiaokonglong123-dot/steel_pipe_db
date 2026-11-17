import { useEffect, useMemo, useRef, useState } from 'react'
import { NavLink, useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { useSession } from '../session'

type Stats = Schemas['DashboardDto']

/* ------------------------------------------------------------------ */
/* 顶栏 + 侧栏                                                         */
/* ------------------------------------------------------------------ */

type NavItem = { to: string; label: string; write?: boolean; admin?: boolean; count?: (s: Stats | undefined) => number; tone?: 'danger' | 'warn' }

const NAV_GROUPS: { title: string; items: NavItem[] }[] = [
  {
    title: '库存',
    items: [
      { to: '/tubes', label: '管子' },
      { to: '/stock-in', label: '入库', write: true },
      { to: '/import', label: '批量导入', write: true },
    ],
  },
  {
    title: '主数据',
    items: [
      { to: '/locations', label: '库位' },
      { to: '/standards', label: '标准规格' },
    ],
  },
  {
    title: '作业',
    items: [
      {
        to: '/checks',
        label: '盘点',
        count: (stats) => stats?.tubes.missing ?? 0,
        tone: 'danger',
      },
    ],
  },
  {
    title: '系统',
    items: [
      { to: '/users', label: '用户', admin: true },
      { to: '/audit', label: '审计链', admin: true },
    ],
  },
]

export function SideNav({ stats, onCommand }: { stats: Stats | undefined; onCommand: () => void }) {
  const { isAdmin, canWrite, user } = useSession()

  return (
    <aside className="sider">
      <button className="sider-item" onClick={onCommand} type="button">
        <span>⌘K</span>
        <span className="count">命令</span>
      </button>
      {NAV_GROUPS.map((group) => {
        const items = group.items.filter((item) => (!item.admin || isAdmin) && (!item.write || canWrite) )
        if (items.length === 0) return null
        return (
          <div key={group.title}>
            <div className="sider-group">{group.title}</div>
            {items.map((item) => {
              const count = item.count?.(stats) ?? 0
              return (
                <NavLink
                  key={item.to}
                  to={item.to}
                  className={({ isActive }) => (isActive ? 'sider-item is-active' : 'sider-item')}
                >
                  <span>{item.label}</span>
                  {count > 0 ? (
                    <span className={`count ${item.tone === 'danger' ? 'is-danger' : item.tone === 'warn' ? 'is-warn' : ''}`}>
                      {count}
                    </span>
                  ) : null}
                </NavLink>
              )
            })}
          </div>
        )
      })}
      {user ? <div className="sider-group">v0.1 · SQLite</div> : null}
    </aside>
  )
}

export function Topbar({ onCommand }: { onCommand: () => void }) {
  const { user, logout } = useSession()
  return (
    <header className="topbar">
      <span className="brand">
        SteelPipe<span className="brand-sub">钢管与筛管库存</span>
      </span>
      <button className="topbar-search" type="button" onClick={onCommand}>
        <span>搜索编号 / 跳转页面</span>
        <kbd>⌘K</kbd>
      </button>
      <span className="topbar-user">
        {user ? (
          <>
            <span>
              {user.display_name} · {user.role === 'admin' ? '管理员' : user.role === 'operator' ? '操作员' : '查看者'}
            </span>
            <button className="btn btn-ghost btn-sm" onClick={() => void logout()}>
              退出
            </button>
          </>
        ) : null}
      </span>
    </header>
  )
}

/* ------------------------------------------------------------------ */
/* KPI                                                                 */
/* ------------------------------------------------------------------ */

export function Kpi({
  label,
  value,
  sub,
  tone,
}: {
  label: string
  value: number | string
  sub?: string
  tone?: 'ok' | 'warn' | 'danger' | 'accent'
}) {
  return (
    <div className={`kpi${tone ? ` is-${tone}` : ''}`}>
      <div className="kpi-label">{label}</div>
      <div className="kpi-value">{value}</div>
      {sub ? <div className="kpi-sub">{sub}</div> : null}
    </div>
  )
}

export function Meter({ value, max }: { value: number; max: number }) {
  const pct = max > 0 ? Math.min(100, Math.round((value / max) * 100)) : 0
  return (
    <div className="meter" title={`${value} / ${max || '∞'}（${pct}%）`}>
      <span style={{ width: `${pct}%` }} />
    </div>
  )
}

/* ------------------------------------------------------------------ */
/* 命令面板                                                            */
/* ------------------------------------------------------------------ */

type Command = { id: string; label: string; hint?: string; run: () => void }

export function CommandPalette({ open, onClose }: { open: boolean; onClose: () => void }) {
  const navigate = useNavigate()
  const { isAdmin, canWrite } = useSession()
  const [query, setQuery] = useState('')
  const [active, setActive] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)

  const commands = useMemo<Command[]>(() => {
    const pages: Command[] = [
      { id: 'dash', label: '仪表盘', hint: '/', run: () => navigate('/') },
      { id: 'tubes', label: '管子列表', run: () => navigate('/tubes') },
      { id: 'checks', label: '盘点', run: () => navigate('/checks') },
      { id: 'locations', label: '库位', run: () => navigate('/locations') },
      { id: 'standards', label: '标准规格', run: () => navigate('/standards') },
    ]
    if (canWrite) {
      pages.push({ id: 'stock-in', label: '入库', run: () => navigate('/stock-in') })
      pages.push({ id: 'import', label: '批量导入', run: () => navigate('/import') })
    }
    if (isAdmin) {
      pages.push({ id: 'users', label: '用户管理', run: () => navigate('/users') })
      pages.push({ id: 'audit', label: '审计链', run: () => navigate('/audit') })
    }
    return pages
  }, [navigate, canWrite, isAdmin])

  const trimmed = query.trim()
  const filtered = useMemo(() => {
    if (!trimmed) return commands
    const lower = trimmed.toLowerCase()
    const hits = commands.filter((command) => command.label.toLowerCase().includes(lower) || command.id.includes(lower))
    const tube: Command = {
      id: 'tube-query',
      label: `打开管子「${trimmed}」`,
      hint: '编号',
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
      <div className="cmd-panel" onMouseDown={(event) => event.stopPropagation()}>
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
          {filtered.map((command, index) => (
            <div
              key={command.id}
              className={index === active ? 'cmd-item is-active' : 'cmd-item'}
              onMouseEnter={() => setActive(index)}
              onMouseDown={() => run(command)}
            >
              <span>{command.label}</span>
              {command.hint ? <span className="hint">{command.hint}</span> : null}
            </div>
          ))}
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
