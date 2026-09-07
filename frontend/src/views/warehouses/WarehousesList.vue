<script setup lang="ts">
import MasterDataCrud from "@/components/common/MasterDataCrud.vue"
import type { MasterColumn } from "@/components/common/MasterDataCrud.vue"
import { listWarehouses, createWarehouse, updateWarehouse, deleteWarehouse } from "@/api/locations"
import type { WarehousePayload } from "@/types/locations"
import type { Page } from "@/types/common"

type Row = Record<string, unknown>

const columns: readonly MasterColumn[] = [
  { prop: "code", label: "编码", required: true },
  { prop: "name", label: "名称", required: true },
  { prop: "address", label: "地址" },
]

async function fetchPage(q: { page: number; page_size: number } & Record<string, string>): Promise<Page<Row>> {
  const r = await listWarehouses({
    page: q.page,
    page_size: q.page_size,
    ...(q.code ? { code: q.code } : {}),
    ...(q.name ? { name: q.name } : {}),
  })
  return r as unknown as Page<Row>
}

const createRow = (row: Row) => createWarehouse(row as unknown as WarehousePayload)
const updateRow = (id: number, row: Row) => updateWarehouse(id, row as unknown as WarehousePayload)
const deleteRow = (id: number) => deleteWarehouse(id)
</script>

<template>
  <MasterDataCrud
    title="仓库"
    subtitle="维护库存仓库主数据"
    :columns="columns"
    :search-fields="['code', 'name']"
    :fetch-page="fetchPage"
    :create-fn="createRow"
    :update-fn="updateRow"
    :delete-fn="deleteRow"
    write-permission="stock.write"
  />
</template>