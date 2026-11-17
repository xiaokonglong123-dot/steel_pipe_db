import { useState } from 'react'
import { useQueryClient } from '@tanstack/react-query'

import { api, ApiError, type Schemas } from '../api/client'
import { PageHead, Problem } from '../components/ui'

export default function ImportPage() {
  const queryClient = useQueryClient()
  const [csv, setCsv] = useState('')
  const [report, setReport] = useState<Schemas['ImportResponse'] | null>(null)
  const [error, setError] = useState<unknown>(null)
  const [busy, setBusy] = useState(false)

  async function upload(event: React.FormEvent) {
    event.preventDefault()
    setBusy(true)
    setError(null)
    try {
      const result = await api<Schemas['ImportResponse']>('/api/tubes/import', {
        method: 'POST',
        raw: csv,
        contentType: 'text/csv',
      })
      setReport(result)
      void queryClient.invalidateQueries({ queryKey: ['tubes'] })
      void queryClient.invalidateQueries({ queryKey: ['locations'] })
    } catch (cause) {
      setReport(null)
      setError(cause)
    } finally {
      setBusy(false)
    }
  }

  async function pickFile(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0]
    if (!file) return
    if (file.size > 20 * 1024 * 1024) {
      setError(new ApiError(413, '文件过大', '上限 20 MiB'))
      return
    }
    setCsv(await file.text())
  }

  return (
    <section>
      <PageHead title="CSV 批量导入" />
      <Problem error={error} />

      <form className="panel" onSubmit={upload}>
        <input type="file" accept=".csv,text/csv" onChange={pickFile} />
        <textarea
          className="textarea"
          rows={8}
          placeholder="或直接粘贴 CSV（首行表头）"
          value={csv}
          onChange={(event) => setCsv(event.target.value)}
        />
        <button className="btn btn-accent" type="submit" disabled={busy || !csv.trim()}>
          {busy ? '导入中…' : '开始导入'}
        </button>
        <p className="field-hint">
          必填列：pipe_id, pipe_type, outer_diameter_mm, wall_thickness_mm, length_m, steel_grade,
          manufacturer, location。单次上限 10 000 行，失败行不影响其他行。
        </p>
      </form>

      {report ? (
        <div className="panel">
          <h2>结果</h2>
          <p>
            共 {report.total} 行 · 成功 {report.inserted} · 失败 {report.failed.length}
          </p>
          {report.failed.length > 0 ? (
            <table className="table">
              <thead><tr><th>行号</th><th>编号</th><th>原因</th></tr></thead>
              <tbody>
                {report.failed.map((row) => (
                  <tr key={`${row.row}-${row.pipe_id}`}>
                    <td className="mono">{row.row}</td>
                    <td className="mono">{row.pipe_id || '—'}</td>
                    <td>{row.error}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : null}
        </div>
      ) : null}
    </section>
  )
}
