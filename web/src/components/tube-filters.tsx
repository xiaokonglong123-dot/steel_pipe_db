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

const STATUS_FILTERS: {
  value: string
  label: string
  countKey?: 'in_stock' | 'in_transfer' | 'missing' | 'out_stock' | 'total'
}[] = [
  { value: '', label: '全部', countKey: 'total' },
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
    <div className="filters-bar">
      <div className="chips" role="group" aria-label="按状态筛选">
        {STATUS_FILTERS.map((option) => (
          <button
            key={option.value || 'all'}
            type="button"
            aria-pressed={state.status === option.value}
            className={state.status === option.value ? 'chip is-active' : 'chip'}
            onClick={() => onChange({ status: option.value })}
          >
            <span>{option.label}</span>
            {option.countKey ? (
              <span className="count">{stats?.tubes[option.countKey] ?? '—'}</span>
            ) : null}
          </button>
        ))}
      </div>

      <span className="spacer" />

      <select
        className="select select-sm filter-select"
        aria-label="按质量筛选"
        value={state.quality}
        onChange={(event) => onChange({ quality: event.target.value })}
      >
        {QUALITY_FILTERS.map((option) => (
          <option key={option.value || 'all'} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>

      <select
        className="select select-sm filter-select"
        aria-label="按库位筛选"
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

      <select
        className="select select-sm filter-select"
        aria-label="按钢级筛选"
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

      {dirty ? (
        <button className="btn btn-ghost btn-sm" type="button" onClick={onReset}>
          清空筛选
        </button>
      ) : null}
    </div>
  )
}