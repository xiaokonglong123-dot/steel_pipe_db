<script setup lang="ts">
import MasterDataCrud from "@/components/common/MasterDataCrud.vue"
import type { MasterColumn } from "@/components/common/MasterDataCrud.vue"
import { listCustomers, createCustomer, updateCustomer, deleteCustomer } from "@/api/parties"
import type { PartyPayload } from "@/types/parties"
import type { Page } from "@/types/common"

type Row = Record<string, unknown>

const columns: readonly MasterColumn[] = [
  { prop: "code", label: "编码", required: true },
  { prop: "name", label: "名称", required: true },
  { prop: "contact", label: "联系人" },
  { prop: "phone", label: "电话" },
  { prop: "email", label: "邮箱" },
  { prop: "address", label: "地址" },
  { prop: "status", label: "状态", formType: "select", options: [
    { value: "active", label: "启用" },
    { value: "inactive", label: "停用" },
  ] },
]

async function fetchPage(q: { page: number; page_size: number } & Record<string, string>): Promise<Page<Row>> {
  const r = await listCustomers({
    page: q.page,
    page_size: q.page_size,
    ...(q.code ? { code: q.code } : {}),
    ...(q.name ? { name: q.name } : {}),
  })
  return r as unknown as Page<Row>
}

const createRow = (row: Row) => createCustomer(row as unknown as PartyPayload)
const updateRow = (id: number, row: Row) => updateCustomer(id, row as unknown as PartyPayload)
const deleteRow = (id: number) => deleteCustomer(id)
</script>

<template>
  <MasterDataCrud
    title="客户"
    subtitle="维护往来客户主数据"
    :columns="columns"
    :search-fields="['code', 'name']"
    :fetch-page="fetchPage"
    :create-fn="createRow"
    :update-fn="updateRow"
    :delete-fn="deleteRow"
    write-permission="customer.write"
  />
</template>