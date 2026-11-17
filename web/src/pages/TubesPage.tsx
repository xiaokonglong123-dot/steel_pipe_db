import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { Empty, Loader, PageHead, Pager, Problem, StatusBadge } from '../components/ui'
import { useSession } from '../session'

type Tube = Schemas['TubeDto']
type Page = Schemas['TubeListResponse']
type LocationList = Schemas['LocationListResponse']

const PAGE_SIZE = 20

export default function TubesPage() {
  const { canWrite } = useSession()
  const queryClient = useQueryClient()
  const navigate = useNavigate()
  const [page, setPage] = useState(1)
  const [status, setStatus] = useState('')
  const [location, setLocation] = useState('')
  const [quality, setQuality] = useState('')
  const [grade, setGrade] = useState('')
  const [selected, setSelected] = useState<string[]>([])
  const [error, setError] = useState<unknown>(null)

  const params = new URLSearchParams({ page: String(page), page_size: String(PAGE_SIZE) })
  if (status) params.set('status', status)
  if (location) params.set('location', location)
  if (quality) params.set('quality_mark', quality)
  if (grade) params.set('steel_grade', grade)

  const tubes = useQuery({
    queryKey: ['tubes', params.toString()],
    queryFn: () => api<Page>(`/api/tubes?${params.toString()}`),
  })
  const locations = useQuery({
    queryKey: ['locations'],
    queryFn: () => api<LocationList>('/api/locations'),
  })

  const batchOut = useMutation({
    mutationFn: () =>
      api<Schemas['BatchStockOutResponse']>('/api/tubes/stock-out/batch', {
        method: 'POST',
        body: { pipe_ids: selected },
      }),
    onSuccess: () => {
      setSelected([])
      setError(null)
      void queryClient.invalidateQueries({ queryKey: ['tubes'] })
      void queryClient.invalidateQueries({ queryKey: ['locations'] })
    },
    onError: (cause) => setError(cause),
  })

  const rows: Tube[] = tubes.data?.tubes ?? []
  const toggle = (pipeId: string) =>
    setSelected((current) =>
      current.includes(pipeId) ? current.filter((id) => id !== pipeId) : [...current, pipeId],
    )

  return (
    <section>
      <PageHead
        title="管子列表"
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

      <div className="toolbar">
        <select className="select" value={status} onChange={(e) => { setStatus(e.target.value); setPage(1) }}>
          <option value="">全部状态</option>
          <option value="in_stock">在库</option>
          <option value="in_transfer">调拨中</option>
          <option value="out_stock">已出库</option>
          <option value="missing">盘亏</option>
        </select>
        <select className="select" value={location} onChange={(e) => { setLocation(e.target.value); setPage(1) }}>
          <option value="">全部库位</option>
          {(locations.data?.locations ?? []).map((row) => (
            <option key={row.code} value={row.code}>
              {row.code}
            </option>
          ))}
        </select>
        <select className="select" value={quality} onChange={(e) => { setQuality(e.target.value); setPage(1) }}>
          <option value="">全部质量</option>
          <option value="normal">正常</option>
          <option value="problematic">问题管</option>
          <option value="scrapped">报废</option>
        </select>
        <input
          className="input input-sm"
          placeholder="钢级"
          value={grade}
          onChange={(e) => { setGrade(e.target.value); setPage(1) }}
        />
        <span className="spacer" />
        {canWrite && selected.length > 0 ? (
          <button className="btn btn-accent" disabled={batchOut.isPending} onClick={() => batchOut.mutate()}>
            批量出库（{selected.length}）
          </button>
        ) : null}
      </div>

      <Problem error={error ?? tubes.error} />
      {tubes.isLoading ? (
        <Loader />
      ) : rows.length === 0 ? (
        <Empty>没有符合条件的管子。</Empty>
      ) : (
        <div className="grid-scroll">
          <table className="table">
            <thead>
              <tr>
                {canWrite ? <th className="col-check" /> : null}
                <th>编号</th>
                <th>类型</th>
                <th>钢级</th>
                <th>规格</th>
                <th>库位</th>
                <th>状态</th>
                <th>质量</th>
                <th>合规</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((tube) => (
                <tr key={tube.pipe_id}>
                  {canWrite ? (
                    <td>
                      <input
                        type="checkbox"
                        checked={selected.includes(tube.pipe_id)}
                        onChange={() => toggle(tube.pipe_id)}
                      />
                    </td>
                  ) : null}
                  <td>
                    <Link className="link" to={`/tubes/${encodeURIComponent(tube.pipe_id)}`}>
                      {tube.pipe_id}
                    </Link>
                  </td>
                  <td>{tube.pipe_type === 'screen_pipe' ? '筛管' : '无缝'}</td>
                  <td>{tube.steel_grade}</td>
                  <td className="mono">
                    {tube.outer_diameter_mm}×{tube.wall_thickness_mm}×{tube.length_m}m
                  </td>
                  <td>{tube.location ?? '—'}</td>
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
                      <StatusBadge value="normal" />
                    ) : (
                      <span className="badge is-warn">强制入库</span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <Pager page={page} pageSize={PAGE_SIZE} total={tubes.data?.total ?? 0} onChange={setPage} />
      <p className="muted">点编号进入详情；勾选后可批量出库（单事务原子）。</p>
      <button className="btn btn-ghost" onClick={() => navigate('/checks')}>
        盘点
      </button>
    </section>
  )
}
