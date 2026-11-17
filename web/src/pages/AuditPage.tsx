import { useState } from 'react'
import { useMutation, useQuery } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { Loader, PageHead, Problem } from '../components/ui'
import { useSession } from '../session'

type Entry = Schemas['AuditEntryDto']

export default function AuditPage() {
  const { isAdmin } = useSession()
  const [action, setAction] = useState('')
  const [error, setError] = useState<unknown>(null)
  const [result, setResult] = useState<string | null>(null)

  const params = new URLSearchParams({ limit: '200' })
  if (action) params.set('action', action)

  const list = useQuery({
    queryKey: ['audit', params.toString()],
    queryFn: () => api<Schemas['AuditListResponse']>(`/api/audit?${params.toString()}`),
    enabled: isAdmin,
  })

  const verify = useMutation({
    mutationFn: () => api<Schemas['VerifyResponse']>('/api/audit/verify', { method: 'POST', body: { batch: 1000 } }),
    onSuccess: (summary) => {
      setError(null)
      setResult(`校验通过：${summary.entries} 条 / ${summary.batches} 批 / tip=${summary.tip.slice(0, 12)}…`)
    },
    onError: (cause) => {
      setResult(null)
      setError(cause)
    },
  })

  if (!isAdmin) return <Problem error={new Error('仅管理员可访问')} />

  return (
    <section>
      <PageHead
        title="审计日志"
        actions={
          <button className="btn btn-subtle" disabled={verify.isPending} onClick={() => verify.mutate()}>
            校验哈希链
          </button>
        }
      />
      <Problem error={error ?? list.error} />
      {result ? <div className="notice">{result}</div> : null}

      <div className="toolbar">
        <input
          className="input input-sm"
          placeholder="按 action 过滤（如 stock_in）"
          value={action}
          onChange={(event) => setAction(event.target.value)}
        />
      </div>

      {list.isLoading ? (
        <Loader />
      ) : (
        <div className="grid-scroll">
          <table className="table">
            <thead><tr><th>#</th><th>时间</th><th>操作人</th><th>动作</th><th>详情</th><th>哈希</th></tr></thead>
            <tbody>
              {(list.data?.entries ?? []).map((entry: Entry) => (
                <tr key={entry.id}>
                  <td className="mono">{entry.id}</td>
                  <td className="mono">{entry.ts}</td>
                  <td className="mono">{entry.user_id ?? '—'}</td>
                  <td>{entry.action}</td>
                  <td className="detail-cell"><code>{entry.detail}</code></td>
                  <td className="mono hash-cell">{entry.current_hash.slice(0, 12)}…</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      <p className="field-hint">共 {list.data?.total ?? 0} 条；列表按 id 升序，取最近 200 条。</p>
    </section>
  )
}
