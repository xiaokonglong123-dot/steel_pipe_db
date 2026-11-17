import { useState } from 'react'
import { Link, useParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { Field, Loader, PageHead, Problem, StatusBadge } from '../components/ui'
import { useSession } from '../session'

type Detail = Schemas['TubeDetailResponse']

export default function TubeDetailPage() {
  const { pipeId = '' } = useParams()
  const queryClient = useQueryClient()
  const { canWrite, isAdmin } = useSession()
  const [error, setError] = useState<unknown>(null)
  const [target, setTarget] = useState('')
  const [mark, setMark] = useState<'normal' | 'problematic' | 'scrapped'>('normal')
  const [note, setNote] = useState('')

  const detail = useQuery({
    queryKey: ['tube', pipeId],
    queryFn: () => api<Detail>(`/api/tubes/${encodeURIComponent(pipeId)}`),
  })
  const locations = useQuery({
    queryKey: ['locations'],
    queryFn: () => api<Schemas['LocationListResponse']>('/api/locations'),
  })

  const refresh = () => {
    void queryClient.invalidateQueries({ queryKey: ['tube', pipeId] })
    void queryClient.invalidateQueries({ queryKey: ['tubes'] })
    void queryClient.invalidateQueries({ queryKey: ['locations'] })
  }

  const act = (path: string, body?: unknown, method: 'POST' | 'DELETE' = 'POST') =>
    useMutation({
      mutationFn: () => api(path, { method, body }),
      onSuccess: () => {
        setError(null)
        refresh()
      },
      onError: (cause) => setError(cause),
    })

  const stockOut = act(`/api/tubes/${encodeURIComponent(pipeId)}/stock-out`, {})
  const transfer = act(`/api/tubes/${encodeURIComponent(pipeId)}/transfer`, { target_location: target })
  const quality = act(`/api/tubes/${encodeURIComponent(pipeId)}/quality`, { quality_mark: mark, note })
  const recover = act(`/api/tubes/${encodeURIComponent(pipeId)}/recover`, {})
  const remove = act(`/api/tubes/${encodeURIComponent(pipeId)}`, undefined, 'DELETE')

  if (detail.isLoading) return <Loader />
  if (detail.error) return <Problem error={detail.error} />
  const tube = detail.data!.tube

  return (
    <section>
      <PageHead
        title={`管子 ${tube.pipe_id}`}
        actions={<Link className="btn btn-subtle" to="/tubes">返回列表</Link>}
      />
      <Problem error={error} />

      <div className="panel">
        <dl className="kv">
          <dt>类型</dt><dd>{tube.pipe_type === 'screen_pipe' ? '筛管' : '无缝钢管'}</dd>
          <dt>规格</dt><dd className="mono">{tube.outer_diameter_mm} × {tube.wall_thickness_mm} mm · {tube.length_m} m</dd>
          <dt>钢级</dt><dd>{tube.steel_grade}</dd>
          <dt>状态</dt><dd><StatusBadge value={tube.status} /></dd>
          <dt>质量</dt><dd><StatusBadge value={tube.quality_mark} /> {tube.quality_note ?? ''}</dd>
          <dt>库位</dt><dd>{tube.location ?? '—'}</dd>
          <dt>厂家</dt><dd>{tube.manufacturer}</dd>
          <dt>批次 / 炉号</dt><dd>{tube.batch_no ?? '—'} / {tube.heat_no ?? '—'}</dd>
          <dt>合规</dt><dd>{tube.compliance_pass === false ? '强制入库（不合规）' : tube.compliance_pass ? '通过' : '—'}</dd>
          <dt>入库时间</dt><dd className="mono">{tube.stock_in_time ?? '—'}</dd>
          <dt>出库时间</dt><dd className="mono">{tube.stock_out_time ?? '—'}</dd>
        </dl>
        {detail.data!.seamless ? (
          <p className="muted">
            无缝参数：表面处理 {detail.data!.seamless.surface_treatment ?? '—'} · 护丝{' '}
            {detail.data!.seamless.end_protector ?? '—'}
          </p>
        ) : null}
        {detail.data!.screen ? (
          <p className="muted">
            筛管参数：槽宽 {detail.data!.screen.slot_width_mm ?? '—'} mm · 过滤精度{' '}
            {detail.data!.screen.filtration_rating_um ?? '—'} μm
          </p>
        ) : null}
      </div>

      {canWrite ? (
        <div className="panel">
          <h2>操作</h2>
          <div className="toolbar">
            <button className="btn btn-accent" disabled={stockOut.isPending} onClick={() => stockOut.mutate()}>
              出库
            </button>
            <input
              className="input input-sm"
              placeholder="目标库位"
              value={target}
              onChange={(event) => setTarget(event.target.value)}
            />
            <button className="btn btn-subtle" disabled={!target || transfer.isPending} onClick={() => transfer.mutate()}>
              调拨
            </button>
            {tube.status === 'missing' ? (
              <button className="btn btn-subtle" onClick={() => recover.mutate()}>
                找回
              </button>
            ) : null}
          </div>
          <div className="toolbar">
            <select className="select" value={mark} onChange={(event) => setMark(event.target.value as typeof mark)}>
              <option value="normal">正常</option>
              <option value="problematic">问题管</option>
              <option value="scrapped">报废</option>
            </select>
            <Field label="备注">
              <input className="input input-sm" value={note} onChange={(event) => setNote(event.target.value)} />
            </Field>
            <button className="btn btn-subtle" onClick={() => quality.mutate()}>
              更新质量标记
            </button>
            {isAdmin ? (
              <button className="btn btn-danger-subtle" onClick={() => { if (confirm('确认软删该管子？')) remove.mutate() }}>
                软删
              </button>
            ) : null}
          </div>
          <p className="field-hint">
            可用库位：{(locations.data?.locations ?? []).map((row) => row.code).join('、') || '—'}
          </p>
        </div>
      ) : null}

      <div className="panel">
        <h2>流转历史</h2>
        {detail.data!.history.length === 0 ? (
          <p className="muted">暂无记录。</p>
        ) : (
          <table className="table">
            <thead>
              <tr><th>单据</th><th>类型</th><th>来源</th><th>去向</th><th>时间</th><th>备注</th></tr>
            </thead>
            <tbody>
              {detail.data!.history.map((row) => (
                <tr key={row.transaction_id}>
                  <td className="mono">#{row.transaction_id}</td>
                  <td>{row.kind}</td>
                  <td>{row.source_location ?? '—'}</td>
                  <td>{row.target_location ?? '—'}</td>
                  <td className="mono">{row.created_at}</td>
                  <td>{row.remark ?? '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </section>
  )
}
