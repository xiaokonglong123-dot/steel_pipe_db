import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { ConfirmDialog, Empty, Field, Loader, PageHead, Problem, useToast } from '../components/ui'
import { useSession } from '../session'

type Standard = Schemas['StandardDto']

export default function StandardsPage() {
  const { isAdmin } = useSession()
  const queryClient = useQueryClient()
  const { push } = useToast()
  const [error, setError] = useState<unknown>(null)
  const [form, setForm] = useState({ grade: '', outer: '', min: '', max: '', source: '' })
  const [removing, setRemoving] = useState<Standard | null>(null)

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
      push('ok', `已新增钢级 ${form.grade.trim()}`)
      setForm({ grade: '', outer: '', min: '', max: '', source: '' })
      setError(null)
      invalidate()
    },
    onError: (cause) => setError(cause),
  })

  const remove = useMutation({
    mutationFn: (row: Standard) => api(`/api/standards/${row.id}`, { method: 'DELETE' }),
    onSuccess: () => {
      push('ok', '已删除规格，审计保留删除前快照')
      setRemoving(null)
      setError(null)
      invalidate()
    },
    onError: (cause) => setError(cause),
  })

  const set =
    (key: keyof typeof form) => (event: React.ChangeEvent<HTMLInputElement>) =>
      setForm((current) => ({ ...current, [key]: event.target.value }))

  const rows = list.data?.standards ?? []

  return (
    <section>
      <PageHead title="API 5CT 标准规格" sub="入库必须匹配已存在的钢级；删除为硬删，审计保留删除前快照" />
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
            <h2>新增规格</h2>
          </div>
          <div className="card-pad form-grid">
            <Field label="钢级">
              <input className="input" value={form.grade} onChange={set('grade')} required />
            </Field>
            <Field label="外径 (mm)">
              <input className="input" value={form.outer} onChange={set('outer')} />
            </Field>
            <Field label="最小壁厚 (mm)">
              <input className="input" value={form.min} onChange={set('min')} />
            </Field>
            <Field label="最大壁厚 (mm)">
              <input className="input" value={form.max} onChange={set('max')} />
            </Field>
            <Field label="来源">
              <input className="input" value={form.source} onChange={set('source')} />
            </Field>
            <div className="form-actions" style={{ alignItems: 'flex-end' }}>
              <button className="btn btn-accent" type="submit" disabled={create.isPending}>
                {create.isPending ? '新增中…' : '新增规格'}
              </button>
            </div>
          </div>
        </form>
      ) : null}

      {list.isLoading ? (
        <Loader />
      ) : rows.length === 0 ? (
        <Empty hint="入库前至少需要一个标准规格">还没有标准规格</Empty>
      ) : (
        <div className="table-wrap">
          <table className="table">
            <thead>
              <tr>
                <th>钢级</th>
                <th className="num">外径 mm</th>
                <th className="num">壁厚区间 mm</th>
                <th className="num">长度公差</th>
                <th>来源</th>
                {isAdmin ? <th className="col-actions" /> : null}
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={row.id}>
                  <td>{row.grade}</td>
                  <td className="num">{row.outer_diameter_mm ?? '—'}</td>
                  <td className="num">
                    {row.min_wall_thickness_mm ?? '—'} ~ {row.max_wall_thickness_mm ?? '—'}
                  </td>
                  <td className="num">{row.length_tolerance_mm ?? '—'}</td>
                  <td className="muted">{row.source ?? '—'}</td>
                  {isAdmin ? (
                    <td className="col-actions">
                      <button className="btn btn-danger btn-sm" type="button" onClick={() => setRemoving(row)}>
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

      <ConfirmDialog
        open={removing !== null}
        title="删除标准规格？"
        body={`将删除钢级 ${removing?.grade ?? ''}。这是硬删，审计会保留删除前快照。`}
        confirmLabel="删除"
        tone="danger"
        pending={remove.isPending}
        onConfirm={() => {
          if (removing) remove.mutate(removing)
        }}
        onCancel={() => setRemoving(null)}
      />
    </section>
  )
}