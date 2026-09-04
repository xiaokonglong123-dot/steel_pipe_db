import { get, page, post, put, del } from "./client"
import type { Page } from "@/types/common"
import type { Supplier, Customer, PartyPayload } from "@/types/parties"

export interface PartyQuery {
  page?: number
  page_size?: number
  code?: string
  name?: string
  status?: string
}

function buildQuery(q: PartyQuery): string {
  const params = new URLSearchParams()
  if (q.page) params.set("page", String(q.page))
  if (q.page_size) params.set("page_size", String(q.page_size))
  if (q.code) params.set("code", q.code)
  if (q.name) params.set("name", q.name)
  if (q.status) params.set("status", q.status)
  const s = params.toString()
  return s ? `?${s}` : ""
}

export async function listSuppliers(q: PartyQuery = {}): Promise<Page<Supplier>> {
  return page<Supplier>(`/suppliers${buildQuery(q)}`)
}
export async function getSupplier(id: number): Promise<Supplier> {
  return get<Supplier>(`/suppliers/${id}`)
}
export async function createSupplier(p: PartyPayload): Promise<Supplier> {
  return post<Supplier>("/suppliers", p)
}
export async function updateSupplier(id: number, p: PartyPayload): Promise<Supplier> {
  return put<Supplier>(`/suppliers/${id}`, p)
}
export async function deleteSupplier(id: number): Promise<void> {
  return del(`/suppliers/${id}`)
}

export async function listCustomers(q: PartyQuery = {}): Promise<Page<Customer>> {
  return page<Customer>(`/customers${buildQuery(q)}`)
}
export async function getCustomer(id: number): Promise<Customer> {
  return get<Customer>(`/customers/${id}`)
}
export async function createCustomer(p: PartyPayload): Promise<Customer> {
  return post<Customer>("/customers", p)
}
export async function updateCustomer(id: number, p: PartyPayload): Promise<Customer> {
  return put<Customer>(`/customers/${id}`, p)
}
export async function deleteCustomer(id: number): Promise<void> {
  return del(`/customers/${id}`)
}