import type { Schemas } from '../api/client'
import { Empty, StatusBadge } from './ui'

type Detail = Schemas['CheckDetailDto']
type Item = Schemas['CheckItemDto']

export type CheckFilter = 'all' | 'pending' | 'ok' | 'missing' | 'moved' | 'surplus'

const FILTER_LABELS: { value: CheckFilter; label: string }[] = [
  { value: 'all', label: '全部' },
  { value: 'pending', label: '未核对' },
  { value: 'ok', label: '一致' },
  { value: 'missing', label: '盘亏' },
  { value: 'moved', label: '位置异常' },
  { value: 'surplus', label: '盘盈' },
]

export function countByFilter(expected: Detail['expected'], items: Item[]): Record<CheckFilter, number> {
  const itemOf = (pipeId: string) => items.find((item) => item.pipe_id === pipeId)
  const counts: Record<CheckFilter, number> = {
    all: expected.length,
    pending: 0,
    ok: 0,
    missing: 0,
    moved: 0,
    surplus: items.filter((item) => item.result === 'surplus').length,
  }
  for (const tube of expected) {
    const item = itemOf(tube.pipe_id)
    if (!item) counts.pending += 1
    else if (item.result === 'ok') counts.ok += 1
    else if (item.result === 'missing') counts.missing += 1
    else if (item.result === 'moved') counts.moved += 1
  }
  return counts
}

export function matchesFilter(
  filter: CheckFilter,
  item: Item | undefined,
  isSurplusRow = false,
): boolean {
  if (filter === 'all') return true
  if (isSurplusRow) return filter === 'surplus'
  if (filter === 'pending') return item === undefined
  return item?.result === filter
}

export function CheckSidebar({
  session,
  counts,
  filter,
  onFilter,
  canWrite,
  isAdmin,
  onConfirm,
  onCancel,
  confirmPending,
  cancelPending,
}: {
  session: Detail['session']
  counts: Record<CheckFilter, number>
  filter: CheckFilter
  onFilter: (next: CheckFilter) => void
  canWrite: boolean
  isAdmin: boolean
  onConfirm: () => void
  onCancel: () => void
  confirmPending: boolean
  cancelPending: boolean
}) {
  const open = session.status === 'open'
  return (
    <aside className="stack aside-card">
      <div className="card">
        <div className="card-head">
          <h2>盘点单</h2>
          <span className="right">
            <StatusBadge value={session.status} />
          </span>
        </div>
        <dl className="kv-grid card-pad" style={{ gridTemplateColumns: '1fr' }}>
          <div>
            <dt>编号</dt>
            <dd className="mono">#{session.id}</dd>
          </div>
          <div>
            <dt>库位</dt>
            <dd className="mono">{session.location}</dd>
          </div>
          <div>
            <dt>开单时间</dt>
            <dd className="mono">{session.created_at.replace('T', ' ').slice(0, 16)}</dd>
          </div>
          {session.remark ? (
            <div>
              <dt>备注</dt>
              <dd>{session.remark}</dd>
            </div>
          ) : null}
        </dl>
      </div>

      <div className="card">
        <div className="card-head">
          <h2>结论分布</h2>
          <span className="sub">{counts.all} 根应盘</span>
        </div>
        <div className="card-pad">
          <div className="chips">
            {FILTER_LABELS.map((option) => (
              <button
                key={option.value}
                type="button"
                aria-pressed={filter === option.value}
                className={filter === option.value ? 'chip is-active' : 'chip'}
                onClick={() => onFilter(option.value)}
              >
                <span>{option.label}</span>
                <span className="count">{counts[option.value]}</span>
              </button>
            ))}
          </div>
        </div>
      </div>

      {open ? (
        <div className="card">
          <div className="card-head">
            <h2>确认</h2>
          </div>
          <div className="card-pad stack" style={{ gap: 10 }}>
            {isAdmin ? (
              <button
                className="btn btn-accent"
                type="button"
                disabled={confirmPending}
                onClick={onConfirm}
              >
                {confirmPending ? '确认中…' : '确认盘点'}
              </button>
            ) : (
              <span className="hint">确认需管理员权限</span>
            )}
            {canWrite ? (
              <button className="btn" type="button" disabled={cancelPending} onClick={onCancel}>
                取消盘点
              </button>
            ) : null}
            <p className="hint" style={{ margin: 0 }}>
              确认在单事务内应用全部差异；任一项失败整体回滚，盘点单保持 open。
            </p>
          </div>
        </div>
      ) : null}
    </aside>
  )
}

export function CheckExpectedTable({
  expected,
  items,
  filter,
  canWrite,
  open,
  activeLocations,
  sessionLocation,
  movedTo,
  onMovedTo,
  onMark,
  pending,
}: {
  expected: Detail['expected']
  items: Item[]
  filter: CheckFilter
  canWrite: boolean
  open: boolean
  activeLocations: string[]
  sessionLocation: string
  movedTo: Record<string, string>
  onMovedTo: (pipeId: string, location: string) => void
  onMark: (payload: { pipe_id: string; result: 'ok' | 'missing' | 'moved'; actual_location?: string }) => void
  pending: boolean
}) {
  const rows = expected.filter((tube) =>
    matchesFilter(filter, items.find((item) => item.pipe_id === tube.pipe_id)),
  )

  if (rows.length === 0) {
    return (
      <div className="card-pad">
        <Empty hint="切换上方的结论筛选">该筛选下没有应盘管子</Empty>
      </div>
    )
  }

  return (
    <div className="table-wrap is-plain" style={{ border: 0, borderRadius: 0 }}>
      <table className="table">
        <thead>
          <tr>
            <th>编号</th>
            <th>钢级</th>
            <th>规格</th>
            <th>质量</th>
            <th>结论</th>
            {canWrite && open ? <th className="col-actions" /> : null}
          </tr>
        </thead>
        <tbody>
          {rows.map((tube) => {
            const item = items.find((row) => row.pipe_id === tube.pipe_id)
            return (
              <tr key={tube.pipe_id} className={!item ? undefined : undefined}>
                <td className="mono">{tube.pipe_id}</td>
                <td>{tube.steel_grade}</td>
                <td className="mono">
                  {tube.outer_diameter_mm}×{tube.wall_thickness_mm}×{tube.length_m}m
                </td>
                <td>
                  <StatusBadge value={tube.quality_mark} />
                </td>
                <td>
                  {item ? <StatusBadge value={item.result} /> : <span className="badge is-none">未核对</span>}
                </td>
                {canWrite && open ? (
                  <td className="col-actions">
                    <div className="row-actions">
                      <button
                        className="btn btn-sm"
                        type="button"
                        disabled={pending}
                        onClick={() => onMark({ pipe_id: tube.pipe_id, result: 'ok' })}
                      >
                        一致
                      </button>
                      <button
                        className="btn btn-danger btn-sm"
                        type="button"
                        disabled={pending}
                        onClick={() => onMark({ pipe_id: tube.pipe_id, result: 'missing' })}
                      >
                        盘亏
                      </button>
                      <select
                        className="select select-sm"
                        style={{ width: 102 }}
                        aria-label="移库目标"
                        value={movedTo[tube.pipe_id] ?? ''}
                        onChange={(event) => onMovedTo(tube.pipe_id, event.target.value)}
                      >
                        <option value="">移库→</option>
                        {activeLocations
                          .filter((code) => code !== sessionLocation)
                          .map((code) => (
                            <option key={code} value={code}>
                              {code}
                            </option>
                          ))}
                      </select>
                      <button
                        className="btn btn-sm"
                        type="button"
                        disabled={!movedTo[tube.pipe_id] || pending}
                        onClick={() =>
                          onMark({
                            pipe_id: tube.pipe_id,
                            result: 'moved',
                            actual_location: movedTo[tube.pipe_id],
                          })
                        }
                      >
                        移库
                      </button>
                    </div>
                  </td>
                ) : null}
              </tr>
            )
          })}
        </tbody>
      </table>
    </div>
  )
}