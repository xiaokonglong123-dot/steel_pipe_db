import { useMemo, useState } from 'react'
import { Link, useParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import {
  CheckExpectedTable,
  CheckSidebar,
  countByFilter,
  matchesFilter,
  type CheckFilter,
} from '../components/check-parts'
import { Field, Loader, PageHead, Problem, StatusBadge } from '../components/ui'
import { useSession } from '../session'

type Detail = Schemas['CheckDetailDto']

export default function CheckDetailPage() {
  const { id = '' } = useParams()
  const { canWrite, isAdmin } = useSession()
  const queryClient = useQueryClient()
  const [error, setError] = useState<unknown>(null)
  const [filter, setFilter] = useState<CheckFilter>('all')
  const [movedTo, setMovedTo] = useState<Record<string, string>>({})
  const [surplus, setSurplus] = useState({ pipe_id: '', wall: '7.0', grade: 'L80' })

  const detail = useQuery({
    queryKey: ['check', id],
    queryFn: () => api<Detail>(`/api/checks/${id}`),
  })

  const refresh = () => {
    void queryClient.invalidateQueries({ queryKey: ['check', id] })
    void queryClient.invalidateQueries({ queryKey: ['tubes'] })
    void queryClient.invalidateQueries({ queryKey: ['locations'] })
    void queryClient.invalidateQueries({ queryKey: ['stats'] })
  }

  const onError = (cause: unknown) => setError(cause)
  const onOk = () => {
    setError(null)
    refresh()
  }

  const mark = useMutation({
    mutationFn: (payload: { pipe_id: string; result: 'ok' | 'missing' | 'moved'; actual_location?: string }) =>
      api(`/api/checks/${id}/items`, { method: 'POST', body: payload }),
    onSuccess: onOk,
    onError,
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
      onOk()
    },
    onError,
  })

  const confirmCheck = useMutation({
    mutationFn: () => api<Schemas['ConfirmResponse']>(`/api/checks/${id}/confirm`, { method: 'POST', body: {} }),
    onSuccess: onOk,
    onError,
  })

  const cancel = useMutation({
    mutationFn: () => api(`/api/checks/${id}/cancel`, { method: 'POST', body: {} }),
    onSuccess: onOk,
    onError,
  })

  const removeItem = useMutation({
    mutationFn: (itemId: number) => api(`/api/checks/${id}/items/${itemId}`, { method: 'DELETE' }),
    onSuccess: onOk,
    onError,
  })

  const counts = useMemo(() => {
    if (!detail.data) {
      return { all: 0, pending: 0, ok: 0, missing: 0, moved: 0, surplus: 0 } as Record<CheckFilter, number>
    }
    return countByFilter(detail.data.expected, detail.data.items)
  }, [detail.data])

  if (detail.isLoading) return <Loader />
  if (detail.error) return <Problem error={detail.error} />
  if (!detail.data) return <Problem error={new Error('盘点单不存在')} />

  const session = detail.data.session
  const items = detail.data.items
  const visibleItems = items.filter((item) => matchesFilter(filter, item, item.result === 'surplus'))

  return (
    <section>
      <PageHead
        title={`盘点 #${session.id} · ${session.location}`}
        actions={
          <Link className="btn btn-subtle" to="/checks">
            返回列表
          </Link>
        }
      />
      <Problem error={error} />

      <div className="split">
        <CheckSidebar
          session={session}
          counts={counts}
          filter={filter}
          onFilter={setFilter}
          canWrite={canWrite}
          isAdmin={isAdmin}
          confirmPending={confirmCheck.isPending}
          cancelPending={cancel.isPending}
          onConfirm={() => {
            if (window.confirm('确认后将原子应用全部差异，继续？')) confirmCheck.mutate()
          }}
          onCancel={() => cancel.mutate()}
        />

        <div className="split-main">
          <h2>应盘清单（{counts.all} 根，未核对 {counts.pending}）</h2>
          <CheckExpectedTable
            expected={detail.data.expected}
            items={items}
            filter={filter}
            canWrite={canWrite}
            open={session.status === 'open'}
            activeLocations={detail.data.active_locations}
            sessionLocation={session.location}
            movedTo={movedTo}
            onMovedTo={(pipeId, location) =>
              setMovedTo((current) => ({ ...current, [pipeId]: location }))
            }
            onMark={(payload) => mark.mutate(payload)}
            pending={mark.isPending}
          />

          {session.status === 'open' && canWrite ? (
            <div className="panel" style={{ marginTop: 10 }}>
              <h2>盘盈登记</h2>
              <div className="toolbar" style={{ marginBottom: 0 }}>
                <Field label="编号">
                  <input
                    className="input input-sm"
                    value={surplus.pipe_id}
                    onChange={(event) => setSurplus({ ...surplus, pipe_id: event.target.value })}
                  />
                </Field>
                <Field label="钢级">
                  <input
                    className="input input-sm"
                    value={surplus.grade}
                    onChange={(event) => setSurplus({ ...surplus, grade: event.target.value })}
                  />
                </Field>
                <Field label="壁厚 (mm)">
                  <input
                    className="input input-sm"
                    value={surplus.wall}
                    onChange={(event) => setSurplus({ ...surplus, wall: event.target.value })}
                  />
                </Field>
                <button
                  className="btn btn-subtle"
                  type="button"
                  disabled={!surplus.pipe_id || addSurplus.isPending}
                  onClick={() => addSurplus.mutate()}
                >
                  登记盘盈
                </button>
              </div>
              <p className="field-hint">盘盈在确认时才走 API 5CT 合规校验；不合规整单确认失败并保持 open。</p>
            </div>
          ) : null}

          <h2>已登记差异（{visibleItems.length} / {items.length}）</h2>
          {visibleItems.length === 0 ? (
            <div className="empty">该筛选下没有已登记条目。</div>
          ) : (
            <div className="grid-scroll">
              <table className="table">
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>结论</th>
                    <th>实际位置</th>
                    {canWrite && session.status === 'open' ? <th className="col-actions" /> : null}
                  </tr>
                </thead>
                <tbody>
                  {visibleItems.map((item) => (
                    <tr key={item.id}>
                      <td className="mono">{item.pipe_id}</td>
                      <td>
                        <StatusBadge value={item.result} />
                      </td>
                      <td className="mono">{item.actual_location ?? '—'}</td>
                      {canWrite && session.status === 'open' ? (
                        <td className="col-actions">
                          <button
                            className="btn btn-ghost btn-sm"
                            type="button"
                            onClick={() => removeItem.mutate(item.id)}
                          >
                            删除
                          </button>
                        </td>
                      ) : null}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>
    </section>
  )
}
