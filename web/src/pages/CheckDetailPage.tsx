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
import {
  ConfirmDialog,
  Empty,
  Field,
  Loader,
  PageHead,
  Problem,
  StatusBadge,
  useToast,
} from '../components/ui'
import { fmtFull } from '../lib/format'
import { useSession } from '../session'

type Detail = Schemas['CheckDetailDto']

export default function CheckDetailPage() {
  const { id = '' } = useParams()
  const { canWrite, isAdmin } = useSession()
  const queryClient = useQueryClient()
  const { push } = useToast()
  const [error, setError] = useState<unknown>(null)
  const [filter, setFilter] = useState<CheckFilter>('all')
  const [movedTo, setMovedTo] = useState<Record<string, string>>({})
  const [surplus, setSurplus] = useState({ pipe_id: '', wall: '7.0', grade: 'L80' })
  const [confirm, setConfirm] = useState<'confirm' | 'cancel' | null>(null)

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
  const onOk = (message?: string) => {
    setError(null)
    setConfirm(null)
    if (message) push('ok', message)
    refresh()
  }

  const mark = useMutation({
    mutationFn: (payload: { pipe_id: string; result: 'ok' | 'missing' | 'moved'; actual_location?: string }) =>
      api(`/api/checks/${id}/items`, { method: 'POST', body: payload }),
    onSuccess: () => onOk(),
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
      onOk('已登记盘盈')
    },
    onError,
  })

  const confirmCheck = useMutation({
    mutationFn: () =>
      api<Schemas['ConfirmResponse']>(`/api/checks/${id}/confirm`, { method: 'POST', body: {} }),
    onSuccess: () => onOk('盘点已确认'),
    onError,
  })

  const cancel = useMutation({
    mutationFn: () => api(`/api/checks/${id}/cancel`, { method: 'POST', body: {} }),
    onSuccess: () => onOk('盘点已取消'),
    onError,
  })

  const removeItem = useMutation({
    mutationFn: (itemId: number) => api(`/api/checks/${id}/items/${itemId}`, { method: 'DELETE' }),
    onSuccess: () => onOk('已删除登记项'),
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
  const open = session.status === 'open'

  return (
    <section>
      <PageHead
        title={`盘点 #${session.id}`}
        sub={
          <>
            库位 <span className="mono">{session.location}</span> · 开单 {fmtFull(session.created_at)}
          </>
        }
        actions={
          <Link className="btn btn-ghost" to="/checks">
            ← 返回列表
          </Link>
        }
      />
      <Problem error={error} />

      <div className="split">
        <div className="stack split-main">
          <div className="card">
            <div className="card-head">
              <h2>应盘清单</h2>
              <span className="sub">
                {counts.all} 根 · 未核对 <b className="num">{counts.pending}</b>
              </span>
            </div>
            <CheckExpectedTable
              expected={detail.data.expected}
              items={items}
              filter={filter}
              canWrite={canWrite}
              open={open}
              activeLocations={detail.data.active_locations}
              sessionLocation={session.location}
              movedTo={movedTo}
              onMovedTo={(pipeId, location) =>
                setMovedTo((current) => ({ ...current, [pipeId]: location }))
              }
              onMark={(payload) => mark.mutate(payload)}
              pending={mark.isPending}
            />
          </div>

          {open && canWrite ? (
            <div className="card">
              <div className="card-head">
                <h2>盘盈登记</h2>
                <span className="sub">新增不在应盘清单中的管子</span>
              </div>
              <form
                className="card-pad form-grid"
                style={{ gridTemplateColumns: 'minmax(0, 1.4fr) repeat(2, minmax(0, 0.8fr)) auto', alignItems: 'end' }}
                onSubmit={(event) => {
                  event.preventDefault()
                  addSurplus.mutate()
                }}
              >
                <Field label="编号">
                  <input
                    className="input"
                    value={surplus.pipe_id}
                    onChange={(event) => setSurplus({ ...surplus, pipe_id: event.target.value })}
                    required
                  />
                </Field>
                <Field label="钢级">
                  <input
                    className="input"
                    value={surplus.grade}
                    onChange={(event) => setSurplus({ ...surplus, grade: event.target.value })}
                  />
                </Field>
                <Field label="壁厚 (mm)">
                  <input
                    className="input"
                    value={surplus.wall}
                    onChange={(event) => setSurplus({ ...surplus, wall: event.target.value })}
                  />
                </Field>
                <button
                  className="btn btn-accent"
                  type="submit"
                  disabled={!surplus.pipe_id || addSurplus.isPending}
                >
                  {addSurplus.isPending ? '登记中…' : '登记'}
                </button>
              </form>
              <p className="hint" style={{ padding: '0 18px 14px' }}>
                盘盈在确认时才走 API 5CT 合规校验；不合规整单确认失败并保持 open。
              </p>
            </div>
          ) : null}

          <div className="card">
            <div className="card-head">
              <h2>已登记差异</h2>
              <span className="sub">
                {visibleItems.length} / {items.length}
              </span>
            </div>
            {visibleItems.length === 0 ? (
              <div className="card-pad">
                <Empty hint="在应盘清单中标记结论后，条目会出现在这里">该筛选下没有已登记条目</Empty>
              </div>
            ) : (
              <div className="table-wrap is-plain" style={{ border: 0, borderRadius: 0 }}>
                <table className="table">
                  <thead>
                    <tr>
                      <th>编号</th>
                      <th>结论</th>
                      <th>实际位置</th>
                      <th>登记时间</th>
                      {canWrite && open ? <th className="col-actions" /> : null}
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
                        <td className="mono">{fmtFull(item.created_at)}</td>
                        {canWrite && open ? (
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

        <CheckSidebar
          session={session}
          counts={counts}
          filter={filter}
          onFilter={setFilter}
          canWrite={canWrite}
          isAdmin={isAdmin}
          confirmPending={confirmCheck.isPending}
          cancelPending={cancel.isPending}
          onConfirm={() => setConfirm('confirm')}
          onCancel={() => setConfirm('cancel')}
        />
      </div>

      <ConfirmDialog
        open={confirm === 'confirm'}
        title="确认盘点？"
        body="确认后将原子应用全部差异：盘亏置为 missing、位置异常改库位、盘盈走合规校验。任一项失败整体回滚。"
        confirmLabel="确认盘点"
        pending={confirmCheck.isPending}
        onConfirm={() => confirmCheck.mutate()}
        onCancel={() => setConfirm(null)}
      />
      <ConfirmDialog
        open={confirm === 'cancel'}
        title="取消盘点？"
        body="盘点单将置为已取消，已登记的差异不会应用。"
        confirmLabel="取消盘点"
        tone="danger"
        pending={cancel.isPending}
        onConfirm={() => cancel.mutate()}
        onCancel={() => setConfirm(null)}
      />
    </section>
  )
}