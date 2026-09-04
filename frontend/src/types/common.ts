// 通用类型 — 与后端 response.rs 对齐
// 成功:  { success:true, request_id, data }
// 分页:  { success:true, request_id, data:{items}, meta:{total,page,page_size,total_pages} }

export type ApiEnvelope<T> = {
  readonly success: boolean
  readonly request_id: string
  readonly data: T
}

export type PageMeta = {
  readonly total: number
  readonly page: number
  readonly page_size: number
  readonly total_pages: number
}

/**
 * 分页列表结果 — api 层把后端 data.items + meta 合并后返回，
 * 供所有列表页统一使用（解决旧 CrudList「total 拿不到」的缺陷）。
 */
export type Page<T> = PageMeta & { readonly items: readonly T[] }

/** @deprecated 旧客户端遗留的扁平分页类型，仅维持类型兼容；新代码请用 Page<T>。 */
export type ApiPage<T> = {
  readonly items: readonly T[]
  readonly total: number
  readonly page: number
  readonly page_size: number
  readonly total_pages: number
}

export type Id = number

export interface PageParams {
  page?: number
  page_size?: number
  [k: string]: string | number | null | undefined
}
