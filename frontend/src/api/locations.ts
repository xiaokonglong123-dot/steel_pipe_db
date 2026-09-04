import { get, page, post, put, del } from "./client"
import type { Page } from "@/types/common"
import type { Warehouse, Location, WarehousePayload, LocationPayload } from "@/types/locations"

export interface ListQuery {
  page?: number
  page_size?: number
  code?: string
  name?: string
}

function build(q: ListQuery): string {
  const p = new URLSearchParams()
  if (q.page) p.set("page", String(q.page))
  if (q.page_size) p.set("page_size", String(q.page_size))
  if (q.code) p.set("code", q.code)
  if (q.name) p.set("name", q.name)
  const s = p.toString()
  return s ? `?${s}` : ""
}

export async function listWarehouses(q: ListQuery = {}): Promise<Page<Warehouse>> {
  return page<Warehouse>(`/warehouses${build(q)}`)
}
export async function getWarehouse(id: number): Promise<Warehouse> {
  return get<Warehouse>(`/warehouses/${id}`)
}
export async function createWarehouse(p: WarehousePayload): Promise<Warehouse> {
  return post<Warehouse>("/warehouses", p)
}
export async function updateWarehouse(id: number, p: WarehousePayload): Promise<Warehouse> {
  return put<Warehouse>(`/warehouses/${id}`, p)
}
export async function deleteWarehouse(id: number): Promise<void> {
  return del(`/warehouses/${id}`)
}

export async function listLocations(q: ListQuery = {}): Promise<Page<Location>> {
  return page<Location>(`/locations${build(q)}`)
}
export async function getLocation(id: number): Promise<Location> {
  return get<Location>(`/locations/${id}`)
}
export async function createLocation(p: LocationPayload): Promise<Location> {
  return post<Location>("/locations", p)
}
export async function updateLocation(id: number, p: LocationPayload): Promise<Location> {
  return put<Location>(`/locations/${id}`, p)
}
export async function deleteLocation(id: number): Promise<void> {
  return del(`/locations/${id}`)
}