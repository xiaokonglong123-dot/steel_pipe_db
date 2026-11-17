import { useState } from 'react'
import { Link, useParams } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { ConfirmDialog, Empty, Field, Loader, PageHead, Problem, StatusBadge, useToast } from '../components/ui'
import { fmtFull } from '../lib/format'
import { useSession } from '../session'

type Detail = Schemas['TubeDetailResponse']

export default function TubeDetailPage() {
  const { pipeId = '' } = useParams()
  const queryClient = useQueryClient()
  const { canWrite, isAdmin } = useSession()
  const { push } = useToast()
  const [error, setError] = useState<unknown>(null)
  const [target, setTarget] = useState('')
  const [mark, setMark] = useState<'normal' | 'problematic' | 'scrapped'>('normal')
  const [note, setNote] = useState('')
  const [confirm, setConfirm] = useState<'out' | 'delete' | null>(null)

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
    void queryClient.invalidateQueries({ queryKey: ['stats'] })
  }

  const onError = (cause: unknown) => setError(cause)
  const done = (message: string) => {
    setError(null)
    setConfirm(null)
    push('ok', message)
    refresh()
  }

  const stockOut = useMutation({
    mutationFn: () => api(`/api/tubes/${encodeURIComponent(pipeId)}/stock-out`, { method: 'POST', body: {} }),
    onSuccess: () => done('已出库'),
    onError,
  })
  const transfer = useMutation({
    mutationFn: () =>
      api(`/api/tubes/${encodeURIComponent(pipeId)}/transfer`, {
        method: 'POST',
        body: { target_location: target },
      }),
    onSuccess: () => {
      setTarget('')
      done('调拨完成')
    },
    onError,
  })
  const quality = useMutation({
    mutationFn: () =>
      api(`/api/tubes/${encodeURIComponent(pipeId)}/quality`, {
        method: 'POST',
        body: { quality_mark: mark, note },
      }),
    onSuccess: () => done('质量标记已更新'),
    onError,
  })
  const recover = useMutation({
    mutationFn: () => api(`/api/tubes/${encodeURIComponent(pipeId)}/recover`, { method: 'POST', body: {} }),
    onSuccess: () => done('已找回'),
    onError,
  })
  const remove = useMutation({
    mutationFn: () => api(`/api/tubes/${encodeURIComponent(pipeId)}`, { method: 'DELETE' }),
    onSuccess: () => done('已软删'),
    onError,
  })

  if (detail.isLoading) return <Loader />
  if (detail.error) return <Problem error={detail.error} />
  const tube = detail.data!.tube

  return (
    <section>
      <PageHead
        title={<span className="mono" style={{ fontSize: 22 }}>{tube.pipe_id}</span>}
        sub={
          <>
            {tube.pipe_type === 'screen_pipe' ? '筛管' : '无缝钢管'} · {tube.steel_grade} ·{' '}
            {tube.outer_diameter_mm}×{tube.wall_thickness_mm} mm
          </>
        }
        actions={
          <Link className="btn btn-ghost" to="/tubes">
            ← 返回列表
          </Link>
        }
      />
      <Problem error={error} />

      <div className="grid-2">
        <div className="stack">
          <div className="card">
            <div className="card-head">
              <h2>基本信息</h2>
              <span className="right">
                <StatusBadge value={tube.status} />
                <StatusBadge value={tube.quality_mark} />
              </span>
            </div>
            <dl className="kv-grid card-pad">
              <div>
                <dt>类型</dt>
                <dd>{tube.pipe_type === 'screen_pipe' ? '筛管' : '无缝钢管'}</dd>
              </div>
              <div>
                <dt>规格</dt>
                <dd className="mono">
                  {tube.outer_diameter_mm} × {tube.wall_thickness_mm} mm · {tube.length_m} m
                </dd>
              </div>
              <div>
                <dt>钢级</dt>
                <dd>{tube.steel_grade}</dd>
              </div>
              <div>
                <dt>库位</dt>
                <dd className="mono">{tube.location ?? '—'}</dd>
              </div>
              <div>
                <dt>厂家</dt>
                <dd>{tube.manufacturer}</dd>
              </div>
              <div>
                <dt>批次 / 炉号</dt>
                <dd className="mono">
                  {tube.batch_no ?? '—'} / {tube.heat_no ?? '—'}
                </dd>
              </div>
              <div>
                <dt>合规</dt>
                <dd>
                  {tube.compliance_pass === false ? (
                    <span className="badge is-warn">强制入库（不合规）</span>
                  ) : tube.compliance_pass ? (
                    <span className="mark is-ok">通过</span>
                  ) : (
                    <span className="dim">—</span>
                  )}
                </dd>
              </div>
              <div>
                <dt>质量备注</dt>
                <dd>{tube.quality_note ?? '—'}</dd>
              </div>
              <div>
                <dt>入库时间</dt>
                <dd className="mono">{fmtFull(tube.stock_in_time)}</dd>
              </div>
              <div>
                <dt>出库时间</dt>
                <dd className="mono">{fmtFull(tube.stock_out_time)}</dd>
              </div>
              {detail.data!.seamless ? (
                <>
                  <div>
                    <dt>表面处理</dt>
                    <dd>{detail.data!.seamless.surface_treatment ?? '—'}</dd>
                  </div>
                  <div>
                    <dt>护丝</dt>
                    <dd>{detail.data!.seamless.end_protector ?? '—'}</dd>
                  </div>
                </>
              ) : null}
              {detail.data!.screen ? (
                <>
                  <div>
                    <dt>槽宽</dt>
                    <dd className="mono">{detail.data!.screen.slot_width_mm ?? '—'} mm</dd>
                  </div>
                  <div>
                    <dt>过滤精度</dt>
                    <dd className="mono">{detail.data!.screen.filtration_rating_um ?? '—'} μm</dd>
                  </div>
                </>
              ) : null}
            </dl>
          </div>

          <div className="card">
            <div className="card-head">
              <h2>流转历史</h2>
              <span className="sub">{detail.data!.history.length} 条</span>
            </div>
            {detail.data!.history.length === 0 ? (
              <div className="card-pad">
                <Empty hint="入库、出库、调拨会生成流转记录">暂无记录</Empty>
              </div>
            ) : (
              <div className="table-wrap is-plain" style={{ border: 0, borderRadius: 0 }}>
                <table className="table">
                  <thead>
                    <tr>
                      <th>单据</th>
                      <th>类型</th>
                      <th>来源</th>
                      <th>去向</th>
                      <th>时间</th>
                      <th>备注</th>
                    </tr>
                  </thead>
                  <tbody>
                    {detail.data!.history.map((row) => (
                      <tr key={row.transaction_id}>
                        <td className="mono">#{row.transaction_id}</td>
                        <td>{row.kind}</td>
                        <td className="mono">{row.source_location ?? '—'}</td>
                        <td className="mono">{row.target_location ?? '—'}</td>
                        <td className="mono">{fmtFull(row.created_at)}</td>
                        <td>{row.remark ?? '—'}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        </div>

        <aside className="stack aside-card">
          {canWrite ? (
            <div className="card">
              <div className="card-head">
                <h2>操作</h2>
              </div>
              <div className="card-pad stack" style={{ gap: 14 }}>
                <div className="toolbar">
                  <button
                    className="btn btn-accent"
                    type="button"
                    disabled={stockOut.isPending || tube.status !== 'in_stock'}
                    onClick={() => setConfirm('out')}
                  >
                    出库
                  </button>
                  {tube.status === 'missing' ? (
                    <button
                      className="btn"
                      type="button"
                      disabled={recover.isPending}
                      onClick={() => recover.mutate()}
                    >
                      找回
                    </button>
                  ) : null}
                </div>

                <div className="field">
                  <span className="field-label">调拨目标库位</span>
                  <div className="toolbar" style={{ gap: 8 }}>
                    <select
                      className="select"
                      value={target}
                      onChange={(event) => setTarget(event.target.value)}
                    >
                      <option value="">选择库位</option>
                      {(locations.data?.locations ?? [])
                        .filter((row) => row.is_active && row.code !== tube.location)
                        .map((row) => (
                          <option key={row.code} value={row.code}>
                            {row.code}（{row.used_count}/{row.capacity ?? '∞'}）
                          </option>
                        ))}
                    </select>
                    <button
                      className="btn"
                      type="button"
                      disabled={!target || transfer.isPending}
                      onClick={() => transfer.mutate()}
                    >
                      调拨
                    </button>
                  </div>
                </div>

                <div className="form-grid" style={{ gap: 10 }}>
                  <Field label="质量标记">
                    <select
                      className="select"
                      value={mark}
                      onChange={(event) => setMark(event.target.value as typeof mark)}
                    >
                      <option value="normal">正常</option>
                      <option value="problematic">问题管</option>
                      <option value="scrapped">报废</option>
                    </select>
                  </Field>
                  <Field label="备注">
                    <input className="input" value={note} onChange={(event) => setNote(event.target.value)} />
                  </Field>
                </div>
                <button
                  className="btn"
                  type="button"
                  disabled={quality.isPending}
                  onClick={() => quality.mutate()}
                >
                  更新质量标记
                </button>

                {isAdmin ? (
                  <div style={{ borderTop: '1px solid var(--line)', paddingTop: 12 }}>
                    <button className="btn btn-danger" type="button" onClick={() => setConfirm('delete')}>
                      软删该管子
                    </button>
                  </div>
                ) : null}
              </div>
            </div>
          ) : null}

          <div className="card">
            <div className="card-head">
              <h2>库位状态</h2>
            </div>
            <div className="card-pad">
              {(locations.data?.locations ?? []).length === 0 ? (
                <span className="dim">无库位</span>
              ) : (
                <div className="stack" style={{ gap: 8 }}>
                  {(locations.data?.locations ?? []).map((row) => (
                    <div key={row.code} style={{ display: 'flex', justifyContent: 'space-between', fontSize: 'var(--fs-sm)' }}>
                      <span className="mono">{row.code}</span>
                      <span className="dim mono">
                        {row.used_count}/{row.capacity ?? '∞'} {row.is_active ? '' : '· 停用'}
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </aside>
      </div>

      <ConfirmDialog
        open={confirm === 'out'}
        title="确认出库？"
        body={`${tube.pipe_id} 将从「在库」变为「已出库」，并写入审计链。`}
        confirmLabel="出库"
        tone="danger"
        pending={stockOut.isPending}
        onConfirm={() => stockOut.mutate()}
        onCancel={() => setConfirm(null)}
      />
      <ConfirmDialog
        open={confirm === 'delete'}
        title="确认软删？"
        body="软删后管子不再出现在默认列表，但审计链保留全部记录。"
        confirmLabel="软删"
        tone="danger"
        pending={remove.isPending}
        onConfirm={() => remove.mutate()}
        onCancel={() => setConfirm(null)}
      />
    </section>
  )
}