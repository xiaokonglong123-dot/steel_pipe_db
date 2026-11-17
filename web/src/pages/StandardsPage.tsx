import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { Field, Loader, PageHead, Problem } from '../components/ui'
import { useSession } from '../session'

type Standard = Schemas['StandardDto']

export default function StandardsPage() {
  const { isAdmin } = useSession()
  const queryClient = useQueryClient()
  const [error, setError] = useState<unknown>(null)
  const [form, setForm] = useState({ grade: '', outer: '', min: '', max: '', source: '' })

  const list = useQuery({
    queryKey: ['standards'],
    queryFn: () => api<Schemas['StandardListResponse']>('/api/standards'),
  })

  const invalidate = () => void queryClient.invalidateQueries({ queryKey: ['standards'] })

  const create = useMutation({
    mutationFn: () =>
      api<Standard>('/api/standards', {
        method: 'POST',
        body: {
          grade: form.grade.trim(),
          outer_diameter_mm: form.outer ? Number(form.outer) : null,
          min_wall_thickness_mm: form.min ? Number(form.min) : null,
          max_wall_thickness_mm: form.max ? Number(form.max) : null,
          length_tolerance_mm: null,
          source: form.source || null,
        },
      }),
    onSuccess: () => {
      setForm({ grade: '', outer: '', min: '', max: '', source: '' })
      setError(null)
      invalidate()
    },
    onError: (cause) => setError(cause),
  })

  const remove = useMutation({
    mutationFn: (row: Standard) => api(`/api/standards/${row.id}`, { method: 'DELETE' }),
    onSuccess: () => {
      setError(null)
      invalidate()
    },
    onError: (cause) => setError(cause),
  })

  const set = (key: keyof typeof form) => (event: React.ChangeEvent<HTMLInputElement>) =>
    setForm((current) => ({ ...current, [key]: event.target.value }))

  return (
    <section>
      <PageHead title="API 5CT 标准规格" />
      <Problem error={error} />

      {isAdmin ? (
        <form
          className="toolbar"
          onSubmit={(event) => {
            event.preventDefault()
            create.mutate()
          }}
        >
          <Field label="钢级"><input className="input input-sm" value={form.grade} onChange={set('grade')} required /></Field>
          <Field label="外径"><input className="input input-sm" value={form.outer} onChange={set('outer')} /></Field>
          <Field label="最小壁厚"><input className="input input-sm" value={form.min} onChange={set('min')} /></Field>
          <Field label="最大壁厚"><input className="input input-sm" value={form.max} onChange={set('max')} /></Field>
          <Field label="来源"><input className="input input-sm" value={form.source} onChange={set('source')} /></Field>
          <button className="btn btn-accent" type="submit" disabled={create.isPending}>新增</button>
        </form>
      ) : null}

      {list.isLoading ? (
        <Loader />
      ) : (
        <table className="table">
          <thead>
            <tr>
              <th>钢级</th><th>外径 (mm)</th><th>壁厚区间 (mm)</th><th>长度公差</th><th>来源</th>
              {isAdmin ? <th>操作</th> : null}
            </tr>
          </thead>
          <tbody>
            {(list.data?.standards ?? []).map((row) => (
              <tr key={row.id}>
                <td>{row.grade}</td>
                <td className="mono">{row.outer_diameter_mm ?? '—'}</td>
                <td className="mono">
                  {row.min_wall_thickness_mm ?? '—'} ~ {row.max_wall_thickness_mm ?? '—'}
                </td>
                <td className="mono">{row.length_tolerance_mm ?? '—'}</td>
                <td>{row.source ?? '—'}</td>
                {isAdmin ? (
                  <td>
                    <button
                      className="btn btn-danger-subtle btn-sm"
                      onClick={() => {
                        if (confirm(`删除规格 ${row.grade}？（审计会留快照）`)) remove.mutate(row)
                      }}
                    >
                      删除
                    </button>
                  </td>
                ) : null}
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <p className="field-hint">入库必须匹配已存在的钢级；删除为硬删，审计保留删除前快照。</p>
    </section>
  )
}
