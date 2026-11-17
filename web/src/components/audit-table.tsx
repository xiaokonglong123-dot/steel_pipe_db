import { useEffect, useRef, useState } from 'react'

import type { Schemas } from '../api/client'

type Entry = Schemas['AuditEntryDto']

const OVERSCAN = 8

/** 读取当前密度下的行高（密度切换会改 CSS 变量）。 */
function useRowHeight(): number {
  const read = () => {
    const raw = getComputedStyle(document.documentElement).getPropertyValue('--row-h').trim()
    const px = Number.parseFloat(raw)
    return Number.isFinite(px) ? px : 34
  }
  const [rowHeight, setRowHeight] = useState(read)

  useEffect(() => {
    const observer = new MutationObserver(() => setRowHeight(read()))
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-density'] })
    return () => observer.disconnect()
  }, [])

  return rowHeight
}

/** 固定行高的窗口化表格：只渲染视口内行，上下用等高空行占位。 */
export function AuditTable({ entries }: { entries: Entry[] }) {
  const viewportRef = useRef<HTMLDivElement>(null)
  const [scrollTop, setScrollTop] = useState(0)
  const [viewportHeight, setViewportHeight] = useState(480)
  const rowHeight = useRowHeight()

  useEffect(() => {
    const element = viewportRef.current
    if (!element) return
    const observer = new ResizeObserver(() => setViewportHeight(element.clientHeight || 480))
    observer.observe(element)
    setViewportHeight(element.clientHeight || 480)
    return () => observer.disconnect()
  }, [])

  const visibleCount = Math.ceil(viewportHeight / rowHeight) + OVERSCAN * 2
  const start = Math.max(0, Math.floor(scrollTop / rowHeight) - OVERSCAN)
  const end = Math.min(entries.length, start + visibleCount)
  const topPad = start * rowHeight
  const bottomPad = Math.max(0, (entries.length - end) * rowHeight)
  const visible = entries.slice(start, end)

  return (
    <div
      ref={viewportRef}
      className="grid-scroll"
      onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
    >
      <table className="table">
        <thead>
          <tr>
            <th className="num">#</th>
            <th>时间</th>
            <th>操作人</th>
            <th>动作</th>
            <th>详情</th>
            <th>哈希</th>
          </tr>
        </thead>
        <tbody>
          {topPad > 0 ? (
            <tr aria-hidden style={{ height: topPad }}>
              <td colSpan={6} style={{ padding: 0, border: 0 }} />
            </tr>
          ) : null}
          {visible.map((entry) => (
            <tr key={entry.id}>
              <td className="num">{entry.id}</td>
              <td className="mono">{entry.ts.replace('T', ' ').slice(0, 19)}</td>
              <td className="mono">{entry.user_id ?? '—'}</td>
              <td>
                <code>{entry.action}</code>
              </td>
              <td className="detail-cell">
                <code>{entry.detail ?? '—'}</code>
              </td>
              <td className="hash-cell mono">{entry.current_hash.slice(0, 12)}…</td>
            </tr>
          ))}
          {bottomPad > 0 ? (
            <tr aria-hidden style={{ height: bottomPad }}>
              <td colSpan={6} style={{ padding: 0, border: 0 }} />
            </tr>
          ) : null}
        </tbody>
      </table>
      {entries.length === 0 ? <div className="empty">没有审计记录。</div> : null}
      <span hidden>{visibleCount}</span>
    </div>
  )
}

export type { Entry as AuditEntry }
