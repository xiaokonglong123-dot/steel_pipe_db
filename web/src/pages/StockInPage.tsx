import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { Field, PageHead, Problem } from '../components/ui'

type StockInBody = Schemas['StockInRequest']

export default function StockInPage() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [error, setError] = useState<unknown>(null)
  const [form, setForm] = useState({
    pipe_id: '',
    pipe_type: 'seamless_pipe' as 'seamless_pipe' | 'screen_pipe',
    outer_diameter_mm: '88.9',
    wall_thickness_mm: '7.0',
    length_m: '9.5',
    steel_grade: 'L80',
    thread_type: 'EUE',
    manufacturer: '',
    batch_no: '',
    location: '',
    quality_mark: 'normal' as 'normal' | 'problematic' | 'scrapped',
    surface_treatment: '',
    slot_width_mm: '',
    filtration_rating_um: '',
  })

  const locations = useQuery({
    queryKey: ['locations'],
    queryFn: () => api<Schemas['LocationListResponse']>('/api/locations'),
  })
  const standards = useQuery({
    queryKey: ['standards'],
    queryFn: () => api<Schemas['StandardListResponse']>('/api/standards'),
  })

  const submit = useMutation({
    mutationFn: (body: StockInBody) =>
      api<Schemas['StockInResponse']>('/api/tubes/stock-in', { method: 'POST', body }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['tubes'] })
      void queryClient.invalidateQueries({ queryKey: ['locations'] })
      navigate('/tubes')
    },
    onError: (cause) => setError(cause),
  })

  const set = (key: keyof typeof form) => (event: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) =>
    setForm((current) => ({ ...current, [key]: event.target.value }))

  function onSubmit(event: React.FormEvent) {
    event.preventDefault()
    setError(null)
    const body: StockInBody = {
      pipe_id: form.pipe_id.trim(),
      pipe_type: form.pipe_type,
      outer_diameter_mm: Number(form.outer_diameter_mm),
      wall_thickness_mm: Number(form.wall_thickness_mm),
      length_m: Number(form.length_m),
      steel_grade: form.steel_grade.trim(),
      thread_type: form.thread_type || null,
      api_5ct_cert: null,
      manufacturer: form.manufacturer.trim(),
      batch_no: form.batch_no || null,
      heat_no: null,
      heat_treat_method: null,
      location: form.location,
      quality_mark: form.quality_mark,
      quality_note: null,
      seamless:
        form.pipe_type === 'seamless_pipe'
          ? { surface_treatment: form.surface_treatment || null, end_protector: null }
          : null,
      screen:
        form.pipe_type === 'screen_pipe'
          ? {
              screen_type: null,
              slot_width_mm: Number(form.slot_width_mm || 0),
              slot_density: null,
              filtration_rating_um: Number(form.filtration_rating_um || 0),
              slot_pattern: null,
              base_pipe_grade: null,
            }
          : null,
      force: false,
    }
    submit.mutate(body)
  }

  return (
    <section>
      <PageHead title="入库" />
      <Problem error={error} />
      <form className="panel form-grid" onSubmit={onSubmit}>
        <Field label="管子编号">
          <input className="input" value={form.pipe_id} onChange={set('pipe_id')} required />
        </Field>
        <Field label="类型">
          <select className="select" value={form.pipe_type} onChange={set('pipe_type')}>
            <option value="seamless_pipe">无缝钢管</option>
            <option value="screen_pipe">筛管</option>
          </select>
        </Field>
        <Field label="外径 (mm)">
          <input className="input" value={form.outer_diameter_mm} onChange={set('outer_diameter_mm')} required />
        </Field>
        <Field label="壁厚 (mm)">
          <input className="input" value={form.wall_thickness_mm} onChange={set('wall_thickness_mm')} required />
        </Field>
        <Field label="长度 (m)">
          <input className="input" value={form.length_m} onChange={set('length_m')} required />
        </Field>
        <Field label="钢级" hint={`已有规格：${(standards.data?.standards ?? []).map((s) => s.grade).join('、') || '—'}`}>
          <input className="input" value={form.steel_grade} onChange={set('steel_grade')} required />
        </Field>
        <Field label="厂家">
          <input className="input" value={form.manufacturer} onChange={set('manufacturer')} required />
        </Field>
        <Field label="库位">
          <select className="select" value={form.location} onChange={set('location')} required>
            <option value="">请选择</option>
            {(locations.data?.locations ?? [])
              .filter((row) => row.is_active)
              .map((row) => (
                <option key={row.code} value={row.code}>
                  {row.code}（{row.used_count}/{row.capacity ?? '∞'}）
                </option>
              ))}
          </select>
        </Field>
        <Field label="质量标记">
          <select className="select" value={form.quality_mark} onChange={set('quality_mark')}>
            <option value="normal">正常</option>
            <option value="problematic">问题管</option>
            <option value="scrapped">报废</option>
          </select>
        </Field>
        <Field label="批次号">
          <input className="input" value={form.batch_no} onChange={set('batch_no')} />
        </Field>
        {form.pipe_type === 'seamless_pipe' ? (
          <Field label="表面处理">
            <input className="input" value={form.surface_treatment} onChange={set('surface_treatment')} />
          </Field>
        ) : (
          <>
            <Field label="槽宽 (mm)">
              <input className="input" value={form.slot_width_mm} onChange={set('slot_width_mm')} required />
            </Field>
            <Field label="过滤精度 (μm)">
              <input className="input" value={form.filtration_rating_um} onChange={set('filtration_rating_um')} required />
            </Field>
          </>
        )}
        <div className="form-actions">
          <button className="btn btn-accent" type="submit" disabled={submit.isPending}>
            {submit.isPending ? '提交中…' : '入库'}
          </button>
        </div>
      </form>
    </section>
  )
}
