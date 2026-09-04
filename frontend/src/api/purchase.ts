import { get, page, post, put, del } from "./client"
import type { Page } from "@/types/common"
import type {
  PurchaseOrder,
  PurchaseOrderDetail,
  PurchaseOrderPayload,
  OrderAction,
} from "@/types/order"

export interface OrderQuery {
  page?: number
  page_size?: number
  supplier_id?: number
  status?: string
  order_no?: string
  order_date_from?: string
  order_date_to?: string
}

function build(q: object): string {
  const p = new URLSearchParams()
  for (const [k, v] of Object.entries(q)) if (v !== undefined && v !== null && v !== "") p.set(k, String(v))
  const s = p.toString()
  return s ? `?${s}` : ""
}

export async function listPurchaseOrders(q: OrderQuery = {}): Promise<Page<PurchaseOrder>> {
  return page<PurchaseOrder>(`/purchase-orders${build(q)}`)
}
export async function getPurchaseOrder(id: number): Promise<PurchaseOrderDetail> {
  return get<PurchaseOrderDetail>(`/purchase-orders/${id}`)
}
export async function createPurchaseOrder(payload: PurchaseOrderPayload): Promise<PurchaseOrder> {
  return post<PurchaseOrder>("/purchase-orders", payload)
}
export async function updatePurchaseOrder(id: number, payload: PurchaseOrderPayload): Promise<PurchaseOrder> {
  return put<PurchaseOrder>(`/purchase-orders/${id}`, payload)
}
export async function deletePurchaseOrder(id: number): Promise<void> {
  return del(`/purchase-orders/${id}`)
}

export async function actPurchaseOrder(id: number, action: Exclude<OrderAction, "edit" | "delete">): Promise<PurchaseOrder> {
  return post<PurchaseOrder>(`/purchase-orders/${id}/${action}`)
}

export interface ReceivedItem {
  item_id: number
  location_id: number
  quantity: string
}
export async function receivePurchaseOrder(id: number, items: readonly ReceivedItem[]): Promise<unknown> {
  return post<unknown>(`/purchase-orders/${id}/receive`, { items })
}