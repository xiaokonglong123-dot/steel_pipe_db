import { get } from "./client"
import { useAuthStore } from "@/stores/auth"
import type {
  FinanceSummaryRow,
  InboundOutboundRow,
  InventorySummaryRow,
  SalesTrendRow,
} from "@/types/reports"

export function inventorySummary(): Promise<readonly InventorySummaryRow[]> {
  return get<readonly InventorySummaryRow[]>("/reports/inventory-summary")
}

export function inboundOutbound(
  q: { item_id?: number; start_date?: string; end_date?: string } = {},
): Promise<readonly InboundOutboundRow[]> {
  const p = new URLSearchParams()
  if (q.item_id != null) p.set("item_id", String(q.item_id))
  if (q.start_date) p.set("start_date", q.start_date)
  if (q.end_date) p.set("end_date", q.end_date)
  const s = p.toString()
  return get<readonly InboundOutboundRow[]>(`/reports/inbound-outbound${s ? `?${s}` : ""}`)
}

export function salesTrend(months = 6): Promise<readonly SalesTrendRow[]> {
  return get<readonly SalesTrendRow[]>(`/reports/sales-trend?months=${months}`)
}

export function financeSummary(): Promise<readonly FinanceSummaryRow[]> {
  return get<readonly FinanceSummaryRow[]>("/reports/finance-summary")
}

/** CSV 导出：后端 ?format=csv 返回原始 CSV 流，需带 auth 直接下载。 */
export async function downloadCsv(path: string, fallbackName: string): Promise<void> {
  const auth = useAuthStore()
  const res = await fetch(`/api${path}`, {
    headers: auth.auth_token ? { Authorization: `Bearer ${auth.auth_token}` } : {},
  })
  if (!res.ok) throw new Error("导出失败")
  const disp = res.headers.get("Content-Disposition") ?? ""
  const m = disp.match(/filename=([^;]+)/)
  const filename = (m ? m[1] : null) ?? fallbackName
  const blob = await res.blob()
  const url = URL.createObjectURL(blob)
  const a = document.createElement("a")
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  a.remove()
  URL.revokeObjectURL(url)
}