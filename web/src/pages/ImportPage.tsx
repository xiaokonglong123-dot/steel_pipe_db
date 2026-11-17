import { useState } from 'react'
import { useQueryClient } from '@tanstack/react-query'

import { api, ApiError, type Schemas } from '../api/client'
import { Empty, PageHead, Problem, useToast } from '../components/ui'

export default function ImportPage() {
  const queryClient = useQueryClient()
  const { push } = useToast()
  const [csv, setCsv] = useState('')
  const [report, setReport] = useState<Schemas['ImportResponse'] | null>(null)
  const [error, setError] = useState<unknown>(null)
  const [busy, setBusy] = useState(false)
  const [fileName, setFileName] = useState('')

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
      push(
        result.failed.length > 0 ? 'danger' : 'ok',
        `导入完成：成功 ${result.inserted} 行，失败 ${result.failed.length} 行`,
      )
      void queryClient.invalidateQueries({ queryKey: ['tubes'] })
      void queryClient.invalidateQueries({ queryKey: ['locations'] })
      void queryClient.invalidateQueries({ queryKey: ['stats'] })
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
    setFileName(file.name)
    setCsv(await file.text())
  }

  return (
    <section>
      <PageHead title="批量导入" sub="CSV 流式解析 · 单次上限 10 000 行 · 失败行不影响其他行" />
      <Problem error={error} />

      <form className="card" onSubmit={upload}>
        <div className="card-head">
          <h2>选择数据源</h2>
          <span className="sub">文件或粘贴</span>
        </div>
        <div className="card-pad stack">
          <label className="file-drop">
            <input type="file" accept=".csv,text/csv" onChange={pickFile} />
            <span>
              {fileName ? `已选择：${fileName}` : '点击选择 CSV 文件，或把文件拖到这里'}
            </span>
            <span className="hint">文件上限 20 MiB</span>
          </label>

          <div className="field">
            <span className="field-label">或直接粘贴 CSV（首行表头）</span>
            <textarea
              className="textarea"
              rows={9}
              placeholder={'pipe_id,pipe_type,outer_diameter_mm,wall_thickness_mm,length_m,steel_grade,manufacturer,location\n…'}
              value={csv}
              onChange={(event) => setCsv(event.target.value)}
            />
          </div>

          <div className="toolbar">
            <button className="btn btn-accent" type="submit" disabled={busy || !csv.trim()}>
              {busy ? '导入中…' : '开始导入'}
            </button>
            <span className="hint">
              必填列：pipe_id · pipe_type · outer_diameter_mm · wall_thickness_mm · length_m · steel_grade ·
              manufacturer · location
            </span>
          </div>
        </div>
      </form>

      {report ? (
        <div className="card" style={{ marginTop: 16 }}>
          <div className="card-head">
            <h2>导入结果</h2>
            <span className="sub">
              共 <b className="num">{report.total}</b> 行 · 成功{' '}
              <b className="num" style={{ color: 'var(--ok)' }}>
                {report.inserted}
              </b>{' '}
              · 失败{' '}
              <b className="num" style={{ color: report.failed.length ? 'var(--danger)' : 'var(--dim)' }}>
                {report.failed.length}
              </b>
            </span>
          </div>
          {report.failed.length === 0 ? (
            <div className="card-pad">
              <Empty>全部行导入成功</Empty>
            </div>
          ) : (
            <div className="table-wrap is-plain" style={{ border: 0, borderRadius: 0 }}>
              <table className="table">
                <thead>
                  <tr>
                    <th className="num">行号</th>
                    <th>编号</th>
                    <th>原因</th>
                  </tr>
                </thead>
                <tbody>
                  {report.failed.map((row) => (
                    <tr key={`${row.row}-${row.pipe_id}`}>
                      <td className="num">{row.row}</td>
                      <td className="mono">{row.pipe_id || '—'}</td>
                      <td className="wrap">{row.error}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      ) : null}
    </section>
  )
}