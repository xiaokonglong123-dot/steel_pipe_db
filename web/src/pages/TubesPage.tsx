import { useMemo, useState } from 'react'
import { Link, useSearchParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { useStats } from '../components/shell'
import { TubeFilters, type TubeFilterState } from '../components/tube-filters'
import { TubeTable } from '../components/tube-table'
import { Empty, Loader, PageHead, Pager, Problem } from '../components/ui'
import { useSession } from '../session'

const PAGE_SIZE = 25

/** 把筛选状态与页码编成查询串；空值不入参，页码 1 省略。 */
function buildQuery(filters: TubeFilterState, page: number): URLSearchParams {
  const query = new URLSearchParams({ page_size: String(PAGE_SIZE) })
  if (filters.status) query.set('status', filters.status)
  if (filters.quality) query.set('quality_mark', filters.quality)
  if (filters.location) query.set('location', filters.location)
  if (filters.grade) query.set('steel_grade', filters.grade)
  if (page > 1) query.set('page', String(page))
  return query
}

export default function TubesPage() {
  const { canWrite } = useSession()
  const queryClient = useQueryClient()
  const stats = useStats()
  const [params, setParams] = useSearchParams()
  const [selected, setSelected] = useState<string[]>([])
  const [error, setError] = useState<unknown>(null)

  const page = Number(params.get('page') ?? 1) || 1
  const filters: TubeFilterState = {
    status: params.get('status') ?? '',
    quality: params.get('quality_mark') ?? '',
    location: params.get('location') ?? '',
    grade: params.get('steel_grade') ?? '',
  }

  function applyFilters(next: TubeFilterState, nextPage = 1) {
    setParams(buildQuery(next, nextPage), { replace: true })
    setSelected([])
  }

  const search = useMemo(
    () => buildQuery(filters, page).toString(),
    [page, filters.status, filters.quality, filters.location, filters.grade],
  )

  const locations = useQuery({
    queryKey: ['locations'],
    queryFn: () => api<Schemas['LocationListResponse']>('/api/locations'),
  })
  const standards = useQuery({
    queryKey: ['standards'],
    queryFn: () => api<Schemas['StandardListResponse']>('/api/standards'),
  })

  const tubes = useQuery({
    queryKey: ['tubes', search],
    queryFn: () => api<Schemas['TubeListResponse']>(`/api/tubes?${search}`),
  })
  const invalidate = () => {
    void queryClient.invalidateQueries({ queryKey: ['tubes'] })
    void queryClient.invalidateQueries({ queryKey: ['locations'] })
    void queryClient.invalidateQueries({ queryKey: ['stats'] })
  }

  const onSuccess = () => {
    setSelected([])
    setError(null)
    invalidate()
  }

  const batchOut = useMutation({
    mutationFn: () =>
      api<Schemas['BatchStockOutResponse']>('/api/tubes/stock-out/batch', {
        method: 'POST',
        body: { pipe_ids: selected },
      }),
    onSuccess,
    onError: (cause) => setError(cause),
  })

  const singleOut = useMutation({
    mutationFn: (pipeId: string) =>
      api(`/api/tubes/${encodeURIComponent(pipeId)}/stock-out`, { method: 'POST', body: {} }),
    onSuccess,
    onError: (cause) => setError(cause),
  })

  const rows = tubes.data?.tubes ?? []
  const toggle = (pipeId: string) =>
    setSelected((current) =>
      current.includes(pipeId) ? current.filter((id) => id !== pipeId) : [...current, pipeId],
    )

  return (
    <section>
      <PageHead
        title="管子"
        actions={
          canWrite ? (
            <>
              <Link className="btn btn-subtle" to="/import">
                批量导入
              </Link>
              <Link className="btn btn-accent" to="/stock-in">
                入库
              </Link>
            </>
          ) : null
        }
      />

      <div className="split">
        <TubeFilters
          state={filters}
          stats={stats}
          locations={locations.data}
          standards={standards.data}
          onChange={(patch) => applyFilters({ ...filters, ...patch })}
          onReset={() => applyFilters({ status: '', quality: '', location: '', grade: '' })}
        />

        <div className="split-main">
          <Problem error={error ?? tubes.error} />

          {tubes.isLoading ? (
            <Loader />
          ) : rows.length === 0 ? (
            <Empty>没有符合条件的管子。</Empty>
          ) : (
            <TubeTable
              rows={rows}
              canWrite={canWrite}
              selected={selected}
              onToggle={toggle}
              onToggleAll={() => setSelected(rows.every((row) => selected.includes(row.pipe_id)) ? [] : rows.map((row) => row.pipe_id))}
              onStockOut={(pipeId) => {
                if (window.confirm(`确认出库 ${pipeId}？`)) singleOut.mutate(pipeId)
              }}
              outPending={singleOut.isPending}
            />
          )}

          {selected.length > 0 ? (
            <div className="bulkbar">
              <span>
                已选 <strong>{selected.length}</strong> 根
              </span>
              <span className="spacer" />
              <button
                className="btn btn-accent"
                type="button"
                disabled={batchOut.isPending}
                onClick={() => batchOut.mutate()}
              >
                {batchOut.isPending ? '出库中…' : '批量出库'}
              </button>
              <button className="btn btn-ghost btn-sm" type="button" onClick={() => setSelected([])}>
                清除
              </button>
            </div>
          ) : null}

          <Pager
            page={page}
            pageSize={PAGE_SIZE}
            total={tubes.data?.total ?? 0}
            onChange={(next) => applyFilters(filters, next)}
          />
          <p className="field-hint">
            行内「出库」为单根操作；勾选后「批量出库」在单事务内原子执行，任一根不满足守卫则整体回滚。
          </p>
        </div>
      </div>
    </section>
  )
}
