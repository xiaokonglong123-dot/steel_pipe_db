import { get, page, post, put, del } from "./client"
import type { Page } from "@/types/common"
import type { Item, ItemPayload } from "@/types/catalog"

export interface ItemQuery {
  page?: number
  page_size?: number
  sku?: string
  name?: string
  status?: string
}

export async function listItems(q: ItemQuery = {}): Promise<Page<Item>> {
  const params = new URLSearchParams()
  if (q.page) params.set("page", String(q.page))
  if (q.page_size) params.set("page_size", String(q.page_size))
  if (q.sku) params.set("sku", q.sku)
  if (q.name) params.set("name", q.name)
  if (q.status) params.set("status", q.status)
  const qs = params.toString()
  return page<Item>(`/items${qs ? `?${qs}` : ""}`)
}

export async function getItem(id: number): Promise<Item> {
  return get<Item>(`/items/${id}`)
}

export async function createItem(payload: ItemPayload): Promise<Item> {
  return post<Item>("/items", payload)
}

export async function updateItem(id: number, payload: ItemPayload): Promise<Item> {
  return put<Item>(`/items/${id}`, payload)
}

export async function deleteItem(id: number): Promise<void> {
  return del(`/items/${id}`)
}