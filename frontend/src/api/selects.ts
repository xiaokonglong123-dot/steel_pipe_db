// api/selects.ts — 为 EntitySelect 提供可搜索的远程选择数据源
// 每个函数返回 Selectable[]（{ id, name, code? }），供下拉展示与回填 id。
import { listItems } from "./catalog"
import { listSuppliers, listCustomers } from "./parties"
import { listWarehouses, listLocations } from "./locations"
import { get } from "./client"
import type { Selectable } from "@/types/select"
import type { Account } from "@/types/finance"

/** 商品：schema = 名称 (SKU) */
export async function searchItems(q: string): Promise<Selectable[]> {
  const res = await listItems({ ...(q ? { name: q } : {}), page_size: 20 })
  return res.items.map((i) => ({ id: i.id, name: i.name, code: i.sku }))
}

/** 供应商 */
export async function searchSuppliers(q: string): Promise<Selectable[]> {
  const res = await listSuppliers({ ...(q ? { name: q, code: q } : {}), page_size: 20 })
  return res.items.map((s) => ({ id: s.id, name: s.name, code: s.code }))
}

/** 客户 */
export async function searchCustomers(q: string): Promise<Selectable[]> {
  const res = await listCustomers({ ...(q ? { name: q, code: q } : {}), page_size: 20 })
  return res.items.map((c) => ({ id: c.id, name: c.name, code: c.code }))
}

/** 库位 */
export async function searchLocations(q: string): Promise<Selectable[]> {
  const res = await listLocations({ ...(q ? { name: q, code: q } : {}), page_size: 20 })
  return res.items.map((l) => ({
    id: l.id,
    name: l.name,
    code: `${l.warehouse_name ?? ""}/${l.code}`,
  }))
}

/** 仓库 */
export async function searchWarehouses(q: string): Promise<Selectable[]> {
  const res = await listWarehouses({ ...(q ? { name: q, code: q } : {}), page_size: 20 })
  return res.items.map((w) => ({ id: w.id, name: w.name, code: w.code }))
}

/** 会计科目：服务端不支持名称搜索，全量拉取后在本地过滤。 */
export async function searchAccounts(q: string): Promise<Selectable[]> {
  const res = await get<readonly Account[]>("/accounts?active_only=true")
  const kw = q.trim().toLowerCase()
  return res
    .filter((a) => !kw || a.name.toLowerCase().includes(kw) || a.code.toLowerCase().includes(kw))
    .map((a) => ({ id: a.id, name: `${a.code} ${a.name}`, code: a.account_type }))
}