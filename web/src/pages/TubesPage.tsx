import { useMemo, useState } from 'react'
import { Link, useSearchParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { useStats } from '../components/shell'
import { TubeFilters, type TubeFilterState } from '../components/tube-filters'
import { TubeTable, type SortKey } from '../components/tube-table'
import { Empty, Loader, PageHead, Pager, Problem } from '../components/ui'
import { useSession } from '../session'

const PAGE_SIZE = 25

type ViewState = TubeFilterState & { sort: SortKey | ''; desc: boolean; page: number }

/** 把视图状态编成查询串：空值不传，页码 1 省略，排序缺省不传。 */
function buildQuery(view: ViewState): URLSearchParams {
  const query = new URLSearchParams({ page_size: String(PAGE_SIZE) })
  if (view.status) query.set('status', view.status)
  if (view.quality) query.set('quality_mark', view.quality)
  if (view.location) query.set('location', view.location)
  if (view.grade) query.set('steel_grade', view.grade)
  if (view.sort) {
    query.set('sort', view.sort)
    query.set('order', view.desc ? 'desc' : 'asc')
  }
  if (view.page > 1) query.set('page', String(view.page))
  return query
}

export default function TubesPage() {
  const { canWrite } = useSession()
  const queryClient = useQueryClient()
  const stats = useStats()
  const [params, setParams] = useSearchParams()
  const [selected, setSelected] = useState<string[]>([])
  const [error, setError] = useState<unknown>(null)

  const sortKey = (params.get('sort') ?? '') as ViewState['sort']
  const view: ViewState = {
    status: params.get('status') ?? '',
    quality: params.get('quality_mark') ?? '',
    location: params.get('location') ?? '',
    grade: params.get('steel_grade') ?? '',
    sort: sortKey,
    desc: params.get('order') === 'desc',
    page: Number(params.get('page') ?? 1) || 1,
  }

  function apply(next: Partial<ViewState>) {
    const merged: ViewState = { ...view, ...next }
    if (!('page' in next)) merged.page = 1
    setParams(buildQuery(merged), { replace: true })
    setSelected([])
  }

  function toggleSort(key: SortKey) {
    const desc = view.sort === key ? !view.desc : false
    apply({ sort: key, desc })
  }

  const search = useMemo(() => buildQuery(view).toString(), [params.toString()])

  const tubes = useQuery({
    queryKey: ['tubes', search],
    queryFn: () => api<Schemas['TubeListResponse']>(`/api/tubes?${search}`),
  })
  const locations = useQuery({
    queryKey: ['locations'],
    queryFn: () => api<Schemas['LocationListResponse']>('/api/locations'),
  })
  const standards = useQuery({
    queryKey: ['standards'],
    queryFn: () => api<Schemas['StandardListResponse']>('/api/standards'),
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
          state={view}
          stats={stats}
          locations={locations.data}
          standards={standards.data}
          onChange={(patch) => apply(patch)}
          onReset={() => apply({ status: '', quality: '', location: '', grade: '' })}
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
              sort={{ key: view.sort, desc: view.desc }}
              onSort={toggleSort}
              onToggle={toggle}
              onToggleAll={() =>
                setSelected(
                  rows.every((row) => selected.includes(row.pipe_id)) ? [] : rows.map((row) => row.pipe_id),
                )
              }
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
            page={view.page}
            pageSize={PAGE_SIZE}
            total={tubes.data?.total ?? 0}
            onChange={(next) => apply({ page: next })}
          />
          <p className="field-hint">
            点表头可排序（编号/钢级/库位/状态/质量）；排序由后端白名单字段执行，翻页稳定。
          </p>
        </div>
      </div>
    </section>
  )
}
