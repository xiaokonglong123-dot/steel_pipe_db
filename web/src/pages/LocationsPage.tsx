import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { Meter } from '../components/shell'
import { Badge, Empty, Field, Loader, PageHead, Problem, useToast } from '../components/ui'
import { useSession } from '../session'

type Location = Schemas['LocationDto']

export default function LocationsPage() {
  const { isAdmin } = useSession()
  const queryClient = useQueryClient()
  const { push } = useToast()
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
    void queryClient.invalidateQueries({ queryKey: ['stats'] })
  }

  const create = useMutation({
    mutationFn: () =>
      api<Location>('/api/locations', {
        method: 'POST',
        body: { code: code.trim(), name: name || null, capacity: capacity ? Number(capacity) : null },
      }),
    onSuccess: () => {
      push('ok', `已创建库位 ${code.trim()}`)
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
    onSuccess: (_data, row) => {
      push('ok', row.is_active ? `已停用 ${row.code}` : `已启用 ${row.code}`)
      setError(null)
      invalidate()
    },
    onError: (cause) => setError(cause),
  })

  const rows = list.data?.locations ?? []

  return (
    <section>
      <PageHead title="库位" sub="停用的库位不能入库，也不能作为调拨目标" />
      <Problem error={error} />

      {isAdmin ? (
        <form
          className="card"
          style={{ marginBottom: 16 }}
          onSubmit={(event) => {
            event.preventDefault()
            create.mutate()
          }}
        >
          <div className="card-head">
            <h2>新建库位</h2>
          </div>
          <div className="card-pad form-grid">
            <Field label="编号">
              <input className="input" value={code} onChange={(event) => setCode(event.target.value)} required />
            </Field>
            <Field label="名称">
              <input className="input" value={name} onChange={(event) => setName(event.target.value)} />
            </Field>
            <Field label="容量（根）" hint="留空表示不限">
              <input className="input" value={capacity} onChange={(event) => setCapacity(event.target.value)} />
            </Field>
            <div className="form-actions" style={{ alignItems: 'flex-end' }}>
              <button className="btn btn-accent" type="submit" disabled={create.isPending}>
                {create.isPending ? '创建中…' : '新建库位'}
              </button>
            </div>
          </div>
        </form>
      ) : null}

      {list.isLoading ? (
        <Loader />
      ) : rows.length === 0 ? (
        <Empty hint={isAdmin ? '用上面的表单创建第一个库位' : '请联系管理员创建库位'}>还没有库位</Empty>
      ) : (
        <div className="table-wrap">
          <table className="table">
            <thead>
              <tr>
                <th>编号</th>
                <th>名称</th>
                <th style={{ width: '34%' }}>占用</th>
                <th>状态</th>
                {isAdmin ? <th className="col-actions" /> : null}
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => {
                const full = Boolean(row.capacity && row.used_count >= row.capacity)
                return (
                  <tr key={row.code}>
                    <td className="mono">{row.code}</td>
                    <td>{row.name ?? '—'}</td>
                    <td>
                      <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                        <div style={{ flex: 1, minWidth: 90 }}>
                          <Meter
                            value={row.used_count}
                            max={row.capacity ?? 0}
                            tone={!row.is_active ? undefined : full ? 'warn' : 'ok'}
                          />
                        </div>
                        <span className="mono dim" style={{ fontSize: 'var(--fs-xs)' }}>
                          {row.used_count} / {row.capacity ?? '∞'}
                        </span>
                      </div>
                    </td>
                    <td>
                      <Badge tone={row.is_active ? 'ok' : 'none'}>{row.is_active ? '启用' : '停用'}</Badge>
                    </td>
                    {isAdmin ? (
                      <td className="col-actions">
                        <button
                          className="btn btn-sm"
                          type="button"
                          disabled={toggle.isPending}
                          onClick={() => toggle.mutate(row)}
                        >
                          {row.is_active ? '停用' : '启用'}
                        </button>
                      </td>
                    ) : null}
                  </tr>
                )
              })}
            </tbody>
          </table>
        </div>
      )}
    </section>
  )
}