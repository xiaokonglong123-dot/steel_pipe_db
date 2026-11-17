import { useCallback, useEffect, useState } from 'react'
import { useMutation } from '@tanstack/react-query'

import { api, type Schemas } from '../api/client'
import { AuditTable, type AuditEntry } from '../components/audit-table'
import { Loader, Notice, PageHead, Problem, useToast } from '../components/ui'

const PAGE = 200

export default function AuditPage() {
  const { push } = useToast()
  const [action, setAction] = useState('')
  const [entries, setEntries] = useState<AuditEntry[]>([])
  const [total, setTotal] = useState(0)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<unknown>(null)
  const [verifyResult, setVerifyResult] = useState<string | null>(null)

  const filterParam = action ? `&action=${encodeURIComponent(action)}` : ''

  const loadFirst = useCallback(async () => {
    setLoading(true)
    setError(null)
    setEntries([])
    setTotal(0)
    try {
      const page = await api<Schemas['AuditListResponse']>(`/api/audit?limit=${PAGE}${filterParam}`)
      setEntries(page.entries)
      setTotal(page.total)
    } catch (cause) {
      setError(cause)
    } finally {
      setLoading(false)
    }
  }, [filterParam])

  useEffect(() => {
    void loadFirst()
  }, [loadFirst])

  async function loadMore() {
    const cursor = entries.at(-1)?.id ?? 0
    setLoading(true)
    setError(null)
    try {
      const page = await api<Schemas['AuditListResponse']>(
        `/api/audit?limit=${PAGE}&after_id=${cursor}${filterParam}`,
      )
      setEntries((current) => [...current, ...page.entries])
      setTotal(page.total)
    } catch (cause) {
      setError(cause)
    } finally {
      setLoading(false)
    }
  }

  const verify = useMutation({
    mutationFn: () =>
      api<Schemas['VerifyResponse']>('/api/audit/verify', { method: 'POST', body: { batch: 1000 } }),
    onSuccess: (summary) => {
      setError(null)
      const text = `校验通过：${summary.entries} 条 / ${summary.batches} 批 / tip=${summary.tip.slice(0, 16)}…`
      setVerifyResult(text)
      push('ok', '审计链校验通过')
    },
    onError: (cause) => {
      setVerifyResult(null)
      setError(cause)
      push('danger', '审计链校验失败')
    },
  })

  const exhausted = entries.length >= total

  return (
    <section>
      <PageHead
        title="审计链"
        sub="sha256(prev‖action‖detail‖ts) · 同事务写入 · 分批校验"
        actions={
          <button
            className="btn btn-accent"
            type="button"
            disabled={verify.isPending}
            onClick={() => verify.mutate()}
          >
            {verify.isPending ? '校验中…' : '校验哈希链'}
          </button>
        }
      />
      <Problem error={error ?? verify.error} />
      {verifyResult ? <Notice>{verifyResult}</Notice> : null}

      <div className="card">
        <div className="card-head">
          <h2>事件流</h2>
          <span className="sub">
            已加载 <b className="num">{entries.length}</b> / {total} 条
          </span>
          <span className="right">
            <input
              className="input"
              style={{ width: 220 }}
              placeholder="按 action 过滤（如 stock_in）"
              value={action}
              onChange={(event) => setAction(event.target.value.trim())}
            />
            {!exhausted ? (
              <button
                className="btn btn-sm"
                type="button"
                disabled={loading}
                onClick={() => void loadMore()}
              >
                {loading ? '加载中…' : '加载更多'}
              </button>
            ) : null}
          </span>
        </div>
        {loading && entries.length === 0 ? <Loader /> : <AuditTable entries={entries} />}
      </div>

      <p className="hint" style={{ marginTop: 12 }}>
        虚拟滚动：只渲染视口内的行，滚动时不重排整表；行高随密度切换自动适配。哈希列截断显示，悬停可看全文。
      </p>
    </section>
  )
}