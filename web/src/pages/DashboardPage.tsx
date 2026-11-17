import { Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { Kpi, Meter, useStats } from '../components/shell'
import { Empty, Loader, PageHead } from '../components/ui'
import { fmtTime, todayLabel } from '../lib/format'
import { useSession } from '../session'

export default function DashboardPage() {
  const { canWrite, isAdmin } = useSession()
  const stats = useStats()

  if (!stats) return <Loader label="汇总中…" />
  const { tubes, locations, today, recent_audit, audit } = stats
  const alarms = tubes.missing + tubes.problematic
  const capacity = locations.capacity || 0

  return (
    <section>
      <PageHead
        title="仪表盘"
        sub={`${todayLabel()} · 数据每 30 秒刷新`}
        actions={
          <>
            {canWrite ? (
              <>
                <Link className="btn" to="/import">
                  批量导入
                </Link>
                <Link className="btn btn-accent" to="/stock-in">
                  入库
                </Link>
              </>
            ) : null}
            {isAdmin ? (
              <Link className="btn btn-ghost" to="/audit">
                审计链
              </Link>
            ) : null}
          </>
        }
      />

      {alarms > 0 ? (
        <div className="alert is-danger" role="alert">
          <span>
            待处理异常：盘亏 {tubes.missing} 根 · 问题管 {tubes.problematic} 根
          </span>
          <span className="right">
            <Link to="/tubes?status=missing">查看盘亏</Link>
            <Link to="/tubes?quality_mark=problematic">查看问题管</Link>
          </span>
        </div>
      ) : null}

      <div className="kpi-grid">
        <Kpi
          hero
          label="在库"
          value={tubes.in_stock}
          unit="根"
          sub={`共 ${tubes.total} 根 · 已出库 ${tubes.out_stock}`}
          meter={{ value: tubes.in_stock, max: capacity || tubes.total }}
          tone="accent"
        />
        <Kpi
          label="调拨中"
          value={tubes.in_transfer}
          unit="根"
          sub={tubes.in_transfer > 0 ? '有在途调拨' : '无在途调拨'}
          tone={tubes.in_transfer > 0 ? 'warn' : undefined}
        />
        <Kpi
          label="盘亏"
          value={tubes.missing}
          unit="根"
          sub={tubes.missing > 0 ? '需要处理' : '无盘亏'}
          tone={tubes.missing > 0 ? 'danger' : undefined}
        />
        <Kpi
          label="问题管"
          value={tubes.problematic}
          unit="根"
          sub={tubes.problematic > 0 ? '需质检复核' : '无问题管'}
          tone={tubes.problematic > 0 ? 'warn' : undefined}
        />
      </div>

      <div className="grid-2">
        <div className="stack">
          <div className="card">
            <div className="card-head">
              <h2>库位占用</h2>
              <span className="sub">
                {locations.used} / {locations.capacity || '∞'} 根 · {locations.active} / {locations.total} 个启用
              </span>
              <span className="right">
                <Meter value={locations.used} max={capacity} />
              </span>
            </div>
            {locations.total === 0 ? (
              <div className="card-pad">
                <Empty hint="先在「库位」页创建库位">还没有库位</Empty>
              </div>
            ) : (
              <div className="loc-list">
                <LocationsList />
              </div>
            )}
          </div>

          <div className="card">
            <div className="card-head">
              <h2>今日流转</h2>
              <span className="sub">按自然日统计</span>
              <span className="right">
                <Link className="hint" to="/audit">
                  查看审计链 →
                </Link>
              </span>
            </div>
            <div className="card-pad grid-3" style={{ gap: 12 }}>
              <TodayStat label="入库" value={today.inbound} tone={today.inbound > 0 ? 'ok' : undefined} />
              <TodayStat label="出库" value={today.outbound} />
              <TodayStat label="调拨" value={today.transfers} />
            </div>
          </div>
        </div>

        <div className="stack">
          <div className="card">
            <div className="card-head">
              <h2>最近活动</h2>
              <span className="right">
                <span className="sub">{audit.entries} 条审计</span>
              </span>
            </div>
            {recent_audit.length === 0 ? (
              <div className="card-pad">
                <Empty hint="入库、出库、盘点等写入后会出现在这里">还没有任何操作</Empty>
              </div>
            ) : (
              <div className="feed">
                {recent_audit
                  .slice()
                  .reverse()
                  .map((row) => (
                    <div className="feed-item" key={row.id}>
                      <span className="feed-time">{fmtTime(row.ts, false)}</span>
                      <span className="feed-dot" />
                      <div className="feed-body">
                        <div>
                          <span className="action">{row.action}</span>
                        </div>
                        <div className="meta">
                          #{row.id} · 操作人 {row.user_id ?? '—'}
                        </div>
                      </div>
                    </div>
                  ))}
              </div>
            )}
          </div>

          <div className="card">
            <div className="card-head">
              <h2>审计链</h2>
              {isAdmin ? (
                <span className="right">
                  <Link className="hint" to="/audit">
                    校验 →
                  </Link>
                </span>
              ) : null}
            </div>
            <dl className="kv-grid card-pad">
              <div>
                <dt>条目</dt>
                <dd className="mono">{audit.entries}</dd>
              </div>
              <div>
                <dt>序号 seq</dt>
                <dd className="mono">{audit.seq}</dd>
              </div>
              <div style={{ gridColumn: '1 / -1' }}>
                <dt>链尾哈希</dt>
                <dd className="mono" style={{ wordBreak: 'break-all', color: 'var(--muted)' }}>
                  {audit.tip}
                </dd>
              </div>
            </dl>
          </div>
        </div>
      </div>

      <p className="hint" style={{ marginTop: 18 }}>
        按 <kbd>⌘K</kbd> 或 <kbd>/</kbd> 打开命令面板，输入编号可直接跳转到管子详情。
      </p>
    </section>
  )
}

function TodayStat({ label, value, tone }: { label: string; value: number; tone?: 'ok' }) {
  return (
    <div className="kpi" style={{ padding: '12px 14px', gap: 2 }}>
      <div className="kpi-label">{label}</div>
      <div className={`kpi-value${tone ? ' is-ok' : ''}`} style={{ fontSize: 26 }}>
        <span>{value}</span>
        <span className="unit">根</span>
      </div>
    </div>
  )
}

/** 库位占用列表：单独查询，避免 Dashboard 汇总 DTO 没有逐库位明细。 */
function LocationsList() {
  const query = useQuery({
    queryKey: ['locations'],
    queryFn: () => api<Schemas['LocationListResponse']>('/api/locations'),
  })
  const rows = query.data?.locations ?? []
  if (!query.data) return <div className="card-pad muted">读取库位…</div>
  return (
    <>
      {rows.map((row) => (
        <div className="loc-row" key={row.code}>
          <span className="loc-code">{row.code}</span>
          <Meter
            value={row.used_count}
            max={row.capacity ?? 0}
            tone={!row.is_active ? undefined : row.capacity && row.used_count >= row.capacity ? 'warn' : 'ok'}
          />
          <span className="loc-count">
            {row.used_count} / {row.capacity ?? '∞'}
            {!row.is_active ? ' · 停用' : ''}
          </span>
        </div>
      ))}
    </>
  )
}