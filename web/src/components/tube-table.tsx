import { Link, useNavigate } from 'react-router-dom'

import type { Schemas } from '../api/client'
import { StatusBadge } from './ui'

type Tube = Schemas['TubeDto']

export function TubeTable({
  rows,
  canWrite,
  selected,
  onToggle,
  onToggleAll,
  onStockOut,
  outPending,
}: {
  rows: Tube[]
  canWrite: boolean
  selected: string[]
  onToggle: (pipeId: string) => void
  onToggleAll: () => void
  onStockOut: (pipeId: string) => void
  outPending: boolean
}) {
  const navigate = useNavigate()
  const allSelected = rows.length > 0 && rows.every((row) => selected.includes(row.pipe_id))

  return (
    <div className="grid-scroll">
      <table className="table">
        <thead>
          <tr>
            {canWrite ? (
              <th className="col-check">
                <input
                  type="checkbox"
                  checked={allSelected}
                  onChange={onToggleAll}
                  aria-label="全选本页"
                />
              </th>
            ) : null}
            <th>编号</th>
            <th>类型</th>
            <th>钢级</th>
            <th>规格</th>
            <th>库位</th>
            <th>状态</th>
            <th>质量</th>
            <th>合规</th>
            <th className="col-actions" />
          </tr>
        </thead>
        <tbody>
          {rows.map((tube) => {
            const isSelected = selected.includes(tube.pipe_id)
            return (
              <tr key={tube.pipe_id} className={isSelected ? 'is-selected' : undefined}>
                {canWrite ? (
                  <td>
                    <input
                      type="checkbox"
                      checked={isSelected}
                      onChange={() => onToggle(tube.pipe_id)}
                      aria-label={`选择 ${tube.pipe_id}`}
                    />
                  </td>
                ) : null}
                <td>
                  <Link className="link mono" to={`/tubes/${encodeURIComponent(tube.pipe_id)}`}>
                    {tube.pipe_id}
                  </Link>
                </td>
                <td className="muted">{tube.pipe_type === 'screen_pipe' ? '筛管' : '无缝'}</td>
                <td>{tube.steel_grade}</td>
                <td className="mono">
                  {tube.outer_diameter_mm}×{tube.wall_thickness_mm}
                </td>
                <td className="mono">{tube.location ?? '—'}</td>
                <td>
                  <StatusBadge value={tube.status} />
                </td>
                <td>
                  <StatusBadge value={tube.quality_mark} />
                </td>
                <td>
                  {tube.compliance_pass === null ? (
                    <span className="muted">—</span>
                  ) : tube.compliance_pass ? (
                    <span className="mark is-normal">通过</span>
                  ) : (
                    <span className="badge is-warn">强制</span>
                  )}
                </td>
                <td className="col-actions">
                  <div className="row-actions">
                    <button
                      className="btn btn-ghost btn-sm"
                      type="button"
                      onClick={() => navigate(`/tubes/${encodeURIComponent(tube.pipe_id)}`)}
                    >
                      详情
                    </button>
                    {canWrite && tube.status === 'in_stock' ? (
                      <button
                        className="btn btn-subtle btn-sm"
                        type="button"
                        disabled={outPending}
                        onClick={() => onStockOut(tube.pipe_id)}
                      >
                        出库
                      </button>
                    ) : null}
                  </div>
                </td>
              </tr>
            )
          })}
        </tbody>
      </table>
    </div>
  )
}
