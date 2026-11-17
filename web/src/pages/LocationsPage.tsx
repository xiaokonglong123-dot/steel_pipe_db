import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { Field, Loader, PageHead, Problem } from '../components/ui'
import { useSession } from '../session'

type Location = Schemas['LocationDto']

export default function LocationsPage() {
  const { isAdmin } = useSession()
  const queryClient = useQueryClient()
  const [code, setCode] = useState('')
  const [name, setName] = useState('')
  const [capacity, setCapacity] = useState('')
  const [error, setError] = useState<unknown>(null)

  const list = useQuery({
    queryKey: ['locations'],
    queryFn: () => api<Schemas['LocationListResponse']>('/api/locations'),
  })

  const invalidate = () => {
    void queryClient.invalidateQueries({ queryKey: ['locations'] })
  }

  const create = useMutation({
    mutationFn: () =>
      api<Location>('/api/locations', {
        method: 'POST',
        body: { code: code.trim(), name: name || null, capacity: capacity ? Number(capacity) : null },
      }),
    onSuccess: () => {
      setCode('')
      setName('')
      setCapacity('')
      setError(null)
      invalidate()
    },
    onError: (cause) => setError(cause),
  })

  const toggle = useMutation({
    mutationFn: (row: Location) =>
      api(`/api/locations/${encodeURIComponent(row.code)}/active`, {
        method: 'POST',
        body: { active: !row.is_active },
      }),
    onSuccess: () => {
      setError(null)
      invalidate()
    },
    onError: (cause) => setError(cause),
  })

  return (
    <section>
      <PageHead title="库位管理" />
      <Problem error={error} />

      {isAdmin ? (
        <form
          className="toolbar"
          onSubmit={(event) => {
            event.preventDefault()
            create.mutate()
          }}
        >
          <Field label="编号"><input className="input input-sm" value={code} onChange={(e) => setCode(e.target.value)} required /></Field>
          <Field label="名称"><input className="input input-sm" value={name} onChange={(e) => setName(e.target.value)} /></Field>
          <Field label="容量"><input className="input input-sm" value={capacity} onChange={(e) => setCapacity(e.target.value)} /></Field>
          <button className="btn btn-accent" type="submit" disabled={create.isPending}>新建</button>
        </form>
      ) : null}

      {list.isLoading ? (
        <Loader />
      ) : (
        <table className="table">
          <thead>
            <tr><th>编号</th><th>名称</th><th>在库/容量</th><th>状态</th>{isAdmin ? <th>操作</th> : null}</tr>
          </thead>
          <tbody>
            {(list.data?.locations ?? []).map((row) => (
              <tr key={row.code}>
                <td className="mono">{row.code}</td>
                <td>{row.name ?? '—'}</td>
                <td className="mono">{row.used_count} / {row.capacity ?? '∞'}</td>
                <td>
                  <span className={`badge ${row.is_active ? 'is-ok' : 'is-none'}`}>
                    {row.is_active ? '启用' : '停用'}
                  </span>
                </td>
                {isAdmin ? (
                  <td>
                    <button className="btn btn-subtle btn-sm" onClick={() => toggle.mutate(row)}>
                      {row.is_active ? '停用' : '启用'}
                    </button>
                  </td>
                ) : null}
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <p className="field-hint">停用的库位不能入库，也不能作为调拨目标。</p>
    </section>
  )
}
