import type { Schemas } from '../api/client'
import { StatusBadge } from './ui'

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
    <aside className="filters">
      <div className="filter-row">
        <h3>盘点单</h3>
        <div className="muted">#{session.id} · {session.location}</div>
        <div style={{ marginTop: 4 }}>
          <StatusBadge value={session.status} />
        </div>
        <div className="field-hint">开单 {session.created_at.slice(0, 16).replace('T', ' ')}</div>
        {session.remark ? <div className="field-hint">备注：{session.remark}</div> : null}
      </div>

      <div className="filter-row">
        <h3>结论</h3>
        {FILTER_LABELS.map((option) => (
          <button
            key={option.value}
            type="button"
            className={filter === option.value ? 'filter-option is-active' : 'filter-option'}
            onClick={() => onFilter(option.value)}
          >
            <span>{option.label}</span>
            <span className="count">{counts[option.value]}</span>
          </button>
        ))}
      </div>

      {open ? (
        <div className="filter-row">
          <h3>操作</h3>
          {isAdmin ? (
            <button
              className="btn btn-accent btn-sm"
              type="button"
              disabled={confirmPending}
              onClick={onConfirm}
            >
              {confirmPending ? '确认中…' : '确认盘点'}
            </button>
          ) : (
            <div className="field-hint">确认需管理员</div>
          )}
          {canWrite ? (
            <button
              className="btn btn-subtle btn-sm"
              type="button"
              disabled={cancelPending}
              onClick={onCancel}
            >
              取消盘点
            </button>
          ) : null}
        </div>
      ) : null}

      <p className="field-hint">确认在单事务内应用全部差异；任一项失败整体回滚，盘点单保持 open。</p>
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
    return <div className="empty">该筛选下没有应盘管子。</div>
  }

  return (
    <div className="grid-scroll">
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
              <tr key={tube.pipe_id}>
                <td className="mono">{tube.pipe_id}</td>
                <td>{tube.steel_grade}</td>
                <td className="mono">
                  {tube.outer_diameter_mm}×{tube.wall_thickness_mm}×{tube.length_m}m
                </td>
                <td>
                  <StatusBadge value={tube.quality_mark} />
                </td>
                <td>
                  {item ? <StatusBadge value={item.result} /> : <span className="muted">未核对</span>}
                </td>
                {canWrite && open ? (
                  <td className="col-actions">
                    <div className="row-actions">
                      <button
                        className="btn btn-subtle btn-sm"
                        type="button"
                        disabled={pending}
                        onClick={() => onMark({ pipe_id: tube.pipe_id, result: 'ok' })}
                      >
                        一致
                      </button>
                      <button
                        className="btn btn-danger-subtle btn-sm"
                        type="button"
                        disabled={pending}
                        onClick={() => onMark({ pipe_id: tube.pipe_id, result: 'missing' })}
                      >
                        盘亏
                      </button>
                      <select
                        className="select select-sm"
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
                        className="btn btn-subtle btn-sm"
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
