import type { Schemas } from '../api/client'

type Stats = Schemas['DashboardDto']
type LocationList = Schemas['LocationListResponse']
type StandardList = Schemas['StandardListResponse']

export type TubeFilterState = {
  status: string
  quality: string
  location: string
  grade: string
}

const STATUS_FILTERS: { value: string; label: string; countKey?: 'in_stock' | 'in_transfer' | 'missing' | 'out_stock' | 'total' }[] = [
  { value: '', label: '全部状态', countKey: 'total' },
  { value: 'in_stock', label: '在库', countKey: 'in_stock' },
  { value: 'in_transfer', label: '调拨中', countKey: 'in_transfer' },
  { value: 'missing', label: '盘亏', countKey: 'missing' },
  { value: 'out_stock', label: '已出库', countKey: 'out_stock' },
]

const QUALITY_FILTERS = [
  { value: '', label: '全部质量' },
  { value: 'normal', label: '正常' },
  { value: 'problematic', label: '问题管' },
  { value: 'scrapped', label: '报废' },
]

export function TubeFilters({
  state,
  stats,
  locations,
  standards,
  onChange,
  onReset,
}: {
  state: TubeFilterState
  stats: Stats | undefined
  locations: LocationList | undefined
  standards: StandardList | undefined
  onChange: (patch: Partial<TubeFilterState>) => void
  onReset: () => void
}) {
  const dirty = Boolean(state.status || state.quality || state.location || state.grade)

  return (
    <aside className="filters">
      <div className="filter-row">
        <h3>状态</h3>
        {STATUS_FILTERS.map((option) => (
          <button
            key={option.value || 'all'}
            type="button"
            className={state.status === option.value ? 'filter-option is-active' : 'filter-option'}
            onClick={() => onChange({ status: option.value })}
          >
            <span>{option.label}</span>
            <span className="count">{option.countKey ? (stats?.tubes[option.countKey] ?? '—') : ''}</span>
          </button>
        ))}
      </div>

      <div className="filter-row">
        <h3>质量</h3>
        {QUALITY_FILTERS.map((option) => (
          <button
            key={option.value || 'all'}
            type="button"
            className={state.quality === option.value ? 'filter-option is-active' : 'filter-option'}
            onClick={() => onChange({ quality: option.value })}
          >
            <span>{option.label}</span>
          </button>
        ))}
      </div>

      <div className="filter-row">
        <h3>库位</h3>
        <select
          className="select select-sm"
          value={state.location}
          onChange={(event) => onChange({ location: event.target.value })}
        >
          <option value="">全部库位</option>
          {(locations?.locations ?? []).map((row) => (
            <option key={row.code} value={row.code}>
              {row.code}（{row.used_count}）
            </option>
          ))}
        </select>
      </div>

      <div className="filter-row">
        <h3>钢级</h3>
        <select
          className="select select-sm"
          value={state.grade}
          onChange={(event) => onChange({ grade: event.target.value })}
        >
          <option value="">全部钢级</option>
          {(standards?.standards ?? []).map((row) => (
            <option key={row.id} value={row.grade}>
              {row.grade}
            </option>
          ))}
        </select>
      </div>

      {dirty ? (
        <button className="btn btn-ghost btn-sm" type="button" onClick={onReset}>
          清空筛选
        </button>
      ) : null}
    </aside>
  )
}
