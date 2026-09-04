import { get, page, post, put } from "./client"
import type { Page } from "@/types/common"
import type {
  AvailableQty,
  CheckDetail,
  CheckSession,
  InboundDetail,
  InboundItemPayload,
  InboundOrder,
  InventoryLog,
  OutboundDetail,
  OutboundItemPayload,
  OutboundOrder,
  StockRow,
} from "@/types/inventory"

export interface Qs {
  page?: number
  page_size?: number
}

function build(q: object): string {
  const p = new URLSearchParams()
  for (const [k, v] of Object.entries(q)) {
    if (v !== undefined && v !== null && v !== "") p.set(k, String(v))
  }
  const s = p.toString()
  return s ? `?${s}` : ""
}

// —— 库存余额 ——
export async function listStock(
  q: Qs & { item_id?: number; location_id?: number; warehouse_id?: number } = {},
): Promise<Page<StockRow>> {
  return page<StockRow>(`/stock${build(q)}`)
}

// —— 流水 ——
export async function listLogs(
  q: Qs & { item_id?: number; location_id?: number; change_type?: string } = {},
): Promise<Page<InventoryLog>> {
  return page<InventoryLog>(`/inventory-logs${build(q)}`)
}

// —— ATP ——
export async function getAvailable(item_id: number, location_id?: number | null): Promise<AvailableQty> {
  return get<AvailableQty>(`/inventory/available${build({ item_id, location_id: location_id ?? undefined })}`)
}

// —— 入库 ——
export async function listInbounds(q: Qs & { status?: string; type?: string } = {}): Promise<Page<InboundOrder>> {
  return page<InboundOrder>(`/inbounds${build(q)}`)
}
export async function getInbound(id: number): Promise<InboundDetail> {
  return get<InboundDetail>(`/inbounds/${id}`)
}
export async function createInbound(payload: {
  inbound_type: string
  order_id?: number | null
  supplier_id?: number | null
  notes?: string | null
  items: readonly InboundItemPayload[]
}): Promise<InboundOrder> {
  return post<InboundOrder>("/inbounds", payload)
}
export async function postInbound(id: number): Promise<InboundOrder> {
  return post<InboundOrder>(`/inbounds/${id}/post`)
}

// —— 出库 ——
export async function listOutbounds(q: Qs & { status?: string; type?: string } = {}): Promise<Page<OutboundOrder>> {
  return page<OutboundOrder>(`/outbounds${build(q)}`)
}
export async function getOutbound(id: number): Promise<OutboundDetail> {
  return get<OutboundDetail>(`/outbounds/${id}`)
}
export async function createOutbound(payload: {
  outbound_type: string
  order_id?: number | null
  customer_id?: number | null
  notes?: string | null
  items: readonly OutboundItemPayload[]
}): Promise<OutboundOrder> {
  return post<OutboundOrder>("/outbounds", payload)
}
export async function postOutbound(id: number): Promise<OutboundOrder> {
  return post<OutboundOrder>(`/outbounds/${id}/post`)
}

// —— 盘点 ——
export async function listChecks(q: Qs = {}): Promise<Page<CheckSession>> {
  return page<CheckSession>(`/inventory/checks${build(q)}`)
}
export async function getCheck(id: number): Promise<CheckDetail> {
  return get<CheckDetail>(`/inventory/checks/${id}`)
}
export async function createCheck(payload: { location_id: number; scope: string }): Promise<CheckSession> {
  return post<CheckSession>("/inventory/checks", payload)
}
export async function recordActual(checkSessionId: number, detailId: number, actualQty: string): Promise<void> {
  return put<void>(`/inventory/checks/${checkSessionId}`, { detail_id: detailId, actual_qty: actualQty })
}
export async function postCheck(id: number): Promise<CheckSession> {
  return post<CheckSession>(`/inventory/checks/${id}/post`)
}