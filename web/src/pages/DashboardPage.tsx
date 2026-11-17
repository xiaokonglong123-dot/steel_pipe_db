import { Link } from 'react-router-dom'

import { Kpi, Meter, useStats } from '../components/shell'
import { Loader, PageHead } from '../components/ui'
import { useSession } from '../session'

export default function DashboardPage() {
  const { canWrite, isAdmin } = useSession()
  const stats = useStats()

  if (!stats) return <Loader label="汇总中…" />
  const { tubes, locations, today, recent_audit, audit } = stats
  const alarms = tubes.missing + tubes.problematic

  return (
    <section>
      <PageHead
        title="仪表盘"
        actions={
          <>
            {canWrite ? (
              <>
                <Link className="btn btn-subtle" to="/import">
                  批量导入
                </Link>
                <Link className="btn btn-accent" to="/stock-in">
                  入库
                </Link>
              </>
            ) : null}
            {isAdmin ? (
              <Link className="btn btn-subtle" to="/audit">
                审计链
              </Link>
            ) : null}
          </>
        }
      />

      {alarms > 0 ? (
        <div className="problem">
          待处理异常：盘亏 {tubes.missing} 根 · 问题管 {tubes.problematic} 根 ·{' '}
          <Link to="/tubes?quality_mark=problematic">查看问题管</Link> ·{' '}
          <Link to="/tubes?status=missing">查看盘亏</Link>
        </div>
      ) : null}

      <div className="kpi-row">
        <Kpi label="在库" value={tubes.in_stock} sub={`共 ${tubes.total} 根`} tone="ok" />
        <Kpi label="调拨中" value={tubes.in_transfer} tone={tubes.in_transfer > 0 ? 'warn' : undefined} />
        <Kpi label="盘亏" value={tubes.missing} tone={tubes.missing > 0 ? 'danger' : undefined} />
        <Kpi label="问题管" value={tubes.problematic} tone={tubes.problematic > 0 ? 'warn' : undefined} />
        <Kpi label="已出库" value={tubes.out_stock} />
        <Kpi label="今日入库" value={today.inbound} tone={today.inbound > 0 ? 'accent' : undefined} />
        <Kpi label="今日出库" value={today.outbound} />
        <Kpi label="今日调拨" value={today.transfers} />
      </div>

      <div className="split">
        <div className="filters">
          <h3>库位占用</h3>
          <div className="muted" style={{ fontSize: 11 }}>
            {locations.used} / {locations.capacity || '∞'} 根
          </div>
          <Meter value={locations.used} max={locations.capacity} />
          <div className="field-hint" style={{ marginTop: 6 }}>
            {locations.active} / {locations.total} 个库位启用
          </div>
          <h3 style={{ marginTop: 12 }}>审计链</h3>
          <div className="mono" style={{ fontSize: 11, color: 'var(--text-muted)' }}>
            {audit.entries} 行 · seq {audit.seq}
          </div>
          <div className="mono" style={{ fontSize: 10.5, color: 'var(--text-dim)', wordBreak: 'break-all' }}>
            {audit.tip.slice(0, 16)}…
          </div>
        </div>

        <div className="split-main">
          <h2>最近审计（6 条）</h2>
          {recent_audit.length === 0 ? (
            <div className="empty">还没有写入任何操作。</div>
          ) : (
            <div className="grid-scroll">
              <table className="table">
                <thead>
                  <tr>
                    <th className="num">#</th>
                    <th>时间</th>
                    <th>操作人</th>
                    <th>动作</th>
                  </tr>
                </thead>
                <tbody>
                  {recent_audit
                    .slice()
                    .reverse()
                    .map((row) => (
                      <tr key={row.id}>
                        <td className="num">{row.id}</td>
                        <td className="mono">{row.ts.replace('T', ' ').slice(0, 19)}</td>
                        <td className="mono">{row.user_id ?? '—'}</td>
                        <td>
                          <code>{row.action}</code>
                        </td>
                      </tr>
                    ))}
                </tbody>
              </table>
            </div>
          )}
          <p className="field-hint">
            按 <kbd>⌘K</kbd> 或 <kbd>/</kbd> 打开命令面板，可直接输入管子编号跳转。
          </p>
        </div>
      </div>
    </section>
  )
}
