import { Link, useNavigate } from 'react-router-dom'

import type { Schemas } from '../api/client'
import { StatusBadge } from './ui'

type Tube = Schemas['TubeDto']

export type SortKey = 'pipe_id' | 'steel_grade' | 'location' | 'status' | 'quality' | 'updated_at'

export type SortState = { key: SortKey | ''; desc: boolean }

function SortHeader({
  label,
  column,
  sort,
  onSort,
}: {
  label: string
  column: SortKey
  sort: SortState
  onSort: (key: SortKey) => void
}) {
  const active = sort.key === column
  return (
    <button className="th-sort" type="button" onClick={() => onSort(column)} aria-sort={active ? (sort.desc ? 'descending' : 'ascending') : undefined}>
      <span>{label}</span>
      {active ? <span className="arrow">{sort.desc ? '▼' : '▲'}</span> : null}
    </button>
  )
}

export function TubeTable({
  rows,
  canWrite,
  selected,
  sort,
  onSort,
  onToggle,
  onToggleAll,
  onStockOut,
  outPending,
}: {
  rows: Tube[]
  canWrite: boolean
  selected: string[]
  sort: SortState
  onSort: (key: SortKey) => void
  onToggle: (pipeId: string) => void
  onToggleAll: () => void
  onStockOut: (pipeId: string) => void
  outPending: boolean
}) {
  const navigate = useNavigate()
  const allSelected = rows.length > 0 && rows.every((row) => selected.includes(row.pipe_id))

  return (
    <div className="table-wrap">
      <table className="table">
        <thead>
          <tr>
            {canWrite ? (
              <th className="col-check">
                <input type="checkbox" checked={allSelected} onChange={onToggleAll} aria-label="全选本页" />
              </th>
            ) : null}
            <th>
              <SortHeader label="编号" column="pipe_id" sort={sort} onSort={onSort} />
            </th>
            <th>类型</th>
            <th>
              <SortHeader label="钢级" column="steel_grade" sort={sort} onSort={onSort} />
            </th>
            <th>规格 mm</th>
            <th>
              <SortHeader label="库位" column="location" sort={sort} onSort={onSort} />
            </th>
            <th>
              <SortHeader label="状态" column="status" sort={sort} onSort={onSort} />
            </th>
            <th>
              <SortHeader label="质量" column="quality" sort={sort} onSort={onSort} />
            </th>
            <th>合规</th>
            <th className="col-actions" />
          </tr>
        </thead>
        <tbody>
          {rows.map((tube) => {
            const isSelected = selected.includes(tube.pipe_id)
            return (
              <tr
                key={tube.pipe_id}
                className={isSelected ? 'is-selected' : undefined}
                onClick={() => navigate(`/tubes/${encodeURIComponent(tube.pipe_id)}`)}
                style={{ cursor: 'pointer' }}
              >
                {canWrite ? (
                  <td onClick={(event) => event.stopPropagation()}>
                    <input
                      type="checkbox"
                      checked={isSelected}
                      onChange={() => onToggle(tube.pipe_id)}
                      aria-label={`选择 ${tube.pipe_id}`}
                    />
                  </td>
                ) : null}
                <td>
                  <Link
                    className="cell-link"
                    to={`/tubes/${encodeURIComponent(tube.pipe_id)}`}
                    onClick={(event) => event.stopPropagation()}
                  >
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
                    <span className="dim">—</span>
                  ) : tube.compliance_pass ? (
                    <span className="mark is-ok">通过</span>
                  ) : (
                    <span className="badge is-warn">强制</span>
                  )}
                </td>
                <td className="col-actions" onClick={(event) => event.stopPropagation()}>
                  <div className="row-actions">
                    {canWrite && tube.status === 'in_stock' ? (
                      <button
                        className="btn btn-sm"
                        type="button"
                        disabled={outPending}
                        onClick={() => onStockOut(tube.pipe_id)}
                      >
                        出库
                      </button>
                    ) : null}
                    <button
                      className="btn btn-ghost btn-sm"
                      type="button"
                      onClick={() => navigate(`/tubes/${encodeURIComponent(tube.pipe_id)}`)}
                    >
                      详情
                    </button>
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