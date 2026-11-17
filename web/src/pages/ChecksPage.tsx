import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { Loader, PageHead, Problem, StatusBadge } from '../components/ui'
import { useSession } from '../session'

export default function ChecksPage() {
  const { canWrite } = useSession()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
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
      void queryClient.invalidateQueries({ queryKey: ['checks'] })
      navigate(`/checks/${session.id}`)
    },
    onError: (cause) => setError(cause),
  })

  return (
    <section>
      <PageHead title="盘点" />
      <Problem error={error} />

      {canWrite ? (
        <form
          className="toolbar"
          onSubmit={(event) => {
            event.preventDefault()
            create.mutate()
          }}
        >
          <select className="select" value={location} onChange={(event) => setLocation(event.target.value)} required>
            <option value="">选择库位</option>
            {(locations.data?.locations ?? [])
              .filter((row) => row.is_active)
              .map((row) => (
                <option key={row.code} value={row.code}>{row.code}</option>
              ))}
          </select>
          <input
            className="input input-sm"
            placeholder="备注（可选）"
            value={remark}
            onChange={(event) => setRemark(event.target.value)}
          />
          <button className="btn btn-accent" type="submit" disabled={!location || create.isPending}>
            开盘点单
          </button>
        </form>
      ) : null}

      {list.isLoading ? (
        <Loader />
      ) : (
        <table className="table">
          <thead>
            <tr><th>#</th><th>库位</th><th>状态</th><th>创建时间</th><th>备注</th></tr>
          </thead>
          <tbody>
            {(list.data?.sessions ?? []).map((session) => (
              <tr key={session.id}>
                <td><Link className="link" to={`/checks/${session.id}`}>#{session.id}</Link></td>
                <td className="mono">{session.location}</td>
                <td><StatusBadge value={session.status} /></td>
                <td className="mono">{session.created_at}</td>
                <td>{session.remark ?? '—'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <p className="field-hint">确认盘点由管理员执行；确认时在单个事务内应用全部差异。</p>
    </section>
  )
}
