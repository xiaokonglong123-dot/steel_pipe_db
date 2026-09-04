import { get, page, post, put, del } from "./client"
import type { Page } from "@/types/common"
import type {
  SalesOrder,
  SalesOrderDetail,
  SalesOrderPayload,
  OrderAction,
} from "@/types/order"

export interface OrderQuery {
  page?: number
  page_size?: number
  customer_id?: number
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

export async function listSalesOrders(q: OrderQuery = {}): Promise<Page<SalesOrder>> {
  return page<SalesOrder>(`/sales-orders${build(q)}`)
}
export async function getSalesOrder(id: number): Promise<SalesOrderDetail> {
  return get<SalesOrderDetail>(`/sales-orders/${id}`)
}
export async function createSalesOrder(payload: SalesOrderPayload): Promise<SalesOrder> {
  return post<SalesOrder>("/sales-orders", payload)
}
export async function updateSalesOrder(id: number, payload: SalesOrderPayload): Promise<SalesOrder> {
  return put<SalesOrder>(`/sales-orders/${id}`, payload)
}
export async function deleteSalesOrder(id: number): Promise<void> {
  return del(`/sales-orders/${id}`)
}

export async function actSalesOrder(id: number, action: Exclude<OrderAction, "edit" | "delete">): Promise<SalesOrder> {
  return post<SalesOrder>(`/sales-orders/${id}/${action}`)
}

export interface ShippedItem {
  item_id: number
  location_id: number
  quantity: string
}
export async function shipSalesOrder(id: number, items: readonly ShippedItem[]): Promise<unknown> {
  return post<unknown>(`/sales-orders/${id}/ship`, { items })
}

export interface ReservationRow {
  readonly id: number
  readonly order_id: number
  readonly sales_order_id?: number
  readonly item_id: number
  readonly location_id: number | null
  readonly quantity: string
}
export async function listReservations(item_id: number): Promise<readonly ReservationRow[]> {
  const r = await get<{ items: readonly ReservationRow[] }>(`/reservations?item_id=${item_id}`)
  return r.items
}