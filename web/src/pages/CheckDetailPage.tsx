import { useState } from 'react'
import { Link, useParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { Field, Loader, PageHead, Problem, StatusBadge } from '../components/ui'
import { useSession } from '../session'

type Detail = Schemas['CheckDetailDto']
type Item = Schemas['CheckItemDto']

export default function CheckDetailPage() {
  const { id = '' } = useParams()
  const { canWrite, isAdmin } = useSession()
  const queryClient = useQueryClient()
  const [error, setError] = useState<unknown>(null)
  const [movedTo, setMovedTo] = useState<Record<string, string>>({})
  const [surplus, setSurplus] = useState({ pipe_id: '', wall: '7.0', grade: 'L80' })

  const detail = useQuery({
    queryKey: ['check', id],
    queryFn: () => api<Detail>(`/api/checks/${id}`),
  })

  const refresh = () => void queryClient.invalidateQueries({ queryKey: ['check', id] })

  const mark = useMutation({
    mutationFn: (payload: { pipe_id: string; result: 'ok' | 'missing' | 'moved'; actual_location?: string }) =>
      api(`/api/checks/${id}/items`, { method: 'POST', body: payload }),
    onSuccess: () => {
      setError(null)
      refresh()
    },
    onError: (cause) => setError(cause),
  })

  const addSurplus = useMutation({
    mutationFn: () =>
      api(`/api/checks/${id}/surplus`, {
        method: 'POST',
        body: {
          pipe_id: surplus.pipe_id.trim(),
          pipe_type: 'seamless_pipe',
          outer_diameter_mm: 88.9,
          wall_thickness_mm: Number(surplus.wall),
          length_m: 9.5,
          steel_grade: surplus.grade.trim(),
          thread_type: null,
          api_5ct_cert: null,
          manufacturer: '盘盈',
          batch_no: null,
          heat_no: null,
          heat_treat_method: null,
          location: detail.data?.session.location ?? '',
          quality_mark: 'normal',
          quality_note: null,
          seamless: null,
          screen: null,
          force: false,
        },
      }),
    onSuccess: () => {
      setSurplus({ pipe_id: '', wall: '7.0', grade: 'L80' })
      setError(null)
      refresh()
    },
    onError: (cause) => setError(cause),
  })

  const confirmCheck = useMutation({
    mutationFn: () => api<Schemas['ConfirmResponse']>(`/api/checks/${id}/confirm`, { method: 'POST', body: {} }),
    onSuccess: () => {
      setError(null)
      refresh()
      void queryClient.invalidateQueries({ queryKey: ['tubes'] })
      void queryClient.invalidateQueries({ queryKey: ['locations'] })
    },
    onError: (cause) => setError(cause),
  })

  const cancel = useMutation({
    mutationFn: () => api(`/api/checks/${id}/cancel`, { method: 'POST', body: {} }),
    onSuccess: () => {
      setError(null)
      refresh()
    },
    onError: (cause) => setError(cause),
  })

  const removeItem = useMutation({
    mutationFn: (itemId: number) => api(`/api/checks/${id}/items/${itemId}`, { method: 'DELETE' }),
    onSuccess: () => {
      setError(null)
      refresh()
    },
    onError: (cause) => setError(cause),
  })

  if (detail.isLoading) return <Loader />
  if (detail.error) return <Problem error={detail.error} />
  const session = detail.data!.session
  const items: Item[] = detail.data!.items
  const itemOf = (pipeId: string) => items.find((item) => item.pipe_id === pipeId)

  return (
    <section>
      <PageHead
        title={`盘点 #${session.id} · ${session.location}`}
        actions={<Link className="btn btn-subtle" to="/checks">返回</Link>}
      />
      <Problem error={error} />

      <div className="toolbar">
        <StatusBadge value={session.status} />
        {session.status === 'open' && isAdmin ? (
          <button
            className="btn btn-accent"
            disabled={confirmCheck.isPending}
            onClick={() => {
              if (window.confirm('确认后将原子应用全部差异，继续？')) confirmCheck.mutate()
            }}
          >
            确认盘点
          </button>
        ) : null}
        {session.status === 'open' && canWrite ? (
          <button className="btn btn-subtle" disabled={cancel.isPending} onClick={() => cancel.mutate()}>
            取消
          </button>
        ) : null}
      </div>

      <h2>应盘清单（系统内 {session.location} 在库）</h2>
      <table className="table">
        <thead>
          <tr><th>编号</th><th>钢级</th><th>规格</th><th>结论</th>{canWrite ? <th>操作</th> : null}</tr>
        </thead>
        <tbody>
          {detail.data!.expected.map((tube) => {
            const item = itemOf(tube.pipe_id)
            return (
              <tr key={tube.pipe_id}>
                <td className="mono">{tube.pipe_id}</td>
                <td>{tube.steel_grade}</td>
                <td className="mono">{tube.outer_diameter_mm}×{tube.wall_thickness_mm}×{tube.length_m}m</td>
                <td>{item ? <StatusBadge value={item.result === 'ok' ? 'in_stock' : item.result === 'missing' ? 'missing' : 'in_transfer'} /> : <span className="muted">未核对</span>}</td>
                {canWrite && session.status === 'open' ? (
                  <td className="row-actions">
                    <button className="btn btn-subtle btn-sm" onClick={() => mark.mutate({ pipe_id: tube.pipe_id, result: 'ok' })}>
                      一致
                    </button>
                    <button className="btn btn-danger-subtle btn-sm" onClick={() => mark.mutate({ pipe_id: tube.pipe_id, result: 'missing' })}>
                      盘亏
                    </button>
                    <select
                      className="select select-sm"
                      value={movedTo[tube.pipe_id] ?? ''}
                      onChange={(event) => setMovedTo((cur) => ({ ...cur, [tube.pipe_id]: event.target.value }))}
                    >
                      <option value="">位置异常→</option>
                      {detail.data!.active_locations
                        .filter((code) => code !== session.location)
                        .map((code) => (
                          <option key={code} value={code}>{code}</option>
                        ))}
                    </select>
                    <button
                      className="btn btn-subtle btn-sm"
                      disabled={!movedTo[tube.pipe_id]}
                      onClick={() =>
                        mark.mutate({
                          pipe_id: tube.pipe_id,
                          result: 'moved',
                          actual_location: movedTo[tube.pipe_id],
                        })
                      }
                    >
                      移库
                    </button>
                  </td>
                ) : null}
              </tr>
            )
          })}
        </tbody>
      </table>

      {session.status === 'open' && canWrite ? (
        <div className="panel">
          <h2>盘盈登记</h2>
          <div className="toolbar">
            <Field label="编号"><input className="input input-sm" value={surplus.pipe_id} onChange={(e) => setSurplus({ ...surplus, pipe_id: e.target.value })} /></Field>
            <Field label="钢级"><input className="input input-sm" value={surplus.grade} onChange={(e) => setSurplus({ ...surplus, grade: e.target.value })} /></Field>
            <Field label="壁厚"><input className="input input-sm" value={surplus.wall} onChange={(e) => setSurplus({ ...surplus, wall: e.target.value })} /></Field>
            <button className="btn btn-subtle" disabled={!surplus.pipe_id || addSurplus.isPending} onClick={() => addSurplus.mutate()}>
              登记盘盈
            </button>
          </div>
          <p className="field-hint">盘盈在确认时才走 API 5CT 合规校验；不合规则整单确认失败并保持 open。</p>
        </div>
      ) : null}

      {items.length > 0 ? (
        <>
          <h2>已登记差异 {items.length} 条</h2>
          <table className="table">
            <thead><tr><th>编号</th><th>结论</th><th>实际位置</th>{canWrite && session.status === 'open' ? <th>操作</th> : null}</tr></thead>
            <tbody>
              {items.map((item) => (
                <tr key={item.id}>
                  <td className="mono">{item.pipe_id}</td>
                  <td>{item.result}</td>
                  <td>{item.actual_location ?? '—'}</td>
                  {canWrite && session.status === 'open' ? (
                    <td>
                      <button className="btn btn-ghost btn-sm" onClick={() => removeItem.mutate(item.id)}>删除</button>
                    </td>
                  ) : null}
                </tr>
              ))}
            </tbody>
          </table>
        </>
      ) : null}
    </section>
  )
}
