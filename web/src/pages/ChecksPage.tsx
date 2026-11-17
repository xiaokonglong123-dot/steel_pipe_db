import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { Empty, Field, Loader, PageHead, Problem, StatusBadge, useToast } from '../components/ui'
import { fmtFull } from '../lib/format'
import { useSession } from '../session'

export default function ChecksPage() {
  const { canWrite } = useSession()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { push } = useToast()
  const [location, setLocation] = useState('')
  const [remark, setRemark] = useState('')
  const [error, setError] = useState<unknown>(null)

  const list = useQuery({
    queryKey: ['checks'],
    queryFn: () => api<Schemas['CheckListResponse']>('/api/checks'),
  })
  const locations = useQuery({
    queryKey: ['locations'],
    queryFn: () => api<Schemas['LocationListResponse']>('/api/locations'),
  })

  const create = useMutation({
    mutationFn: () =>
      api<Schemas['CheckSessionDto']>('/api/checks', {
        method: 'POST',
        body: { location, remark: remark || null },
      }),
    onSuccess: (session) => {
      push('ok', `已开盘点单 #${session.id}`)
      void queryClient.invalidateQueries({ queryKey: ['checks'] })
      navigate(`/checks/${session.id}`)
    },
    onError: (cause) => setError(cause),
  })

  const sessions = list.data?.sessions ?? []

  return (
    <section>
      <PageHead title="盘点" sub="确认盘点由管理员执行；确认时在单个事务内应用全部差异" />
      <Problem error={error} />

      {canWrite ? (
        <form
          className="card"
          style={{ marginBottom: 16 }}
          onSubmit={(event) => {
            event.preventDefault()
            create.mutate()
          }}
        >
          <div className="card-head">
            <h2>开盘点单</h2>
          </div>
          <div className="card-pad form-grid">
            <Field label="库位">
              <select
                className="select"
                value={location}
                onChange={(event) => setLocation(event.target.value)}
                required
              >
                <option value="">选择库位</option>
                {(locations.data?.locations ?? [])
                  .filter((row) => row.is_active)
                  .map((row) => (
                    <option key={row.code} value={row.code}>
                      {row.code}（{row.used_count} 根）
                    </option>
                  ))}
              </select>
            </Field>
            <Field label="备注">
              <input
                className="input"
                placeholder="可选"
                value={remark}
                onChange={(event) => setRemark(event.target.value)}
              />
            </Field>
            <div className="form-actions" style={{ alignItems: 'flex-end' }}>
              <button className="btn btn-accent" type="submit" disabled={!location || create.isPending}>
                {create.isPending ? '创建中…' : '开单'}
              </button>
            </div>
          </div>
        </form>
      ) : null}

      {list.isLoading ? (
        <Loader />
      ) : sessions.length === 0 ? (
        <Empty hint={canWrite ? '用上面的表单开第一张盘点单' : '还没有盘点单'}>还没有盘点单</Empty>
      ) : (
        <div className="table-wrap">
          <table className="table">
            <thead>
              <tr>
                <th className="num">#</th>
                <th>库位</th>
                <th>状态</th>
                <th>创建时间</th>
                <th>备注</th>
                <th className="col-actions" />
              </tr>
            </thead>
            <tbody>
              {sessions.map((session) => (
                <tr
                  key={session.id}
                  style={{ cursor: 'pointer' }}
                  onClick={() => navigate(`/checks/${session.id}`)}
                >
                  <td className="num">{session.id}</td>
                  <td className="mono">{session.location}</td>
                  <td>
                    <StatusBadge value={session.status} />
                  </td>
                  <td className="mono">{fmtFull(session.created_at)}</td>
                  <td className="muted">{session.remark ?? '—'}</td>
                  <td className="col-actions" onClick={(event) => event.stopPropagation()}>
                    <Link className="btn btn-sm" to={`/checks/${session.id}`}>
                      打开
                    </Link>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  )
}