<script setup lang="ts">
import { onMounted, reactive, ref } from "vue"
import { ElMessage } from "element-plus"
import { DataTable, EntitySelect, PageHeader, SearchBar } from "@/components"
import { listLogs } from "@/api/inventory"
import { searchItems, searchLocations } from "@/api/selects"
import type { InventoryLog } from "@/types/inventory"

const rows = ref<InventoryLog[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const loading = ref(false)
const filters = reactive<{ item_id: number | null; location_id: number | null; change_type: string }>({
  item_id: null, location_id: null, change_type: "",
})

const changeTypeOptions = [
  { value: "inbound", label: "入库" },
  { value: "outbound", label: "出库" },
  { value: "check_adjust", label: "盘点调整" },
]

function changeTypeTag(t: string): "success" | "warning" | "info" {
  if (t === "inbound") return "success"
  if (t === "outbound") return "warning"
  return "info"
}

function changeTypeLabel(t: string): string {
  return changeTypeOptions.find((o) => o.value === t)?.label ?? t
}

function signedQuantity(row: InventoryLog): string {
  return row.change_type === "inbound" ? `+${row.quantity}` : `-${row.quantity}`
}

async function load(): Promise<void> {
  loading.value = true
  try {
    const r = await listLogs({
      page: page.value,
      page_size: pageSize.value,
      ...(filters.item_id != null ? { item_id: filters.item_id } : {}),
      ...(filters.location_id != null ? { location_id: filters.location_id } : {}),
      ...(filters.change_type ? { change_type: filters.change_type } : {}),
    })
    rows.value = [...r.items]
    total.value = r.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

function reset(): void {
  filters.item_id = null
  filters.location_id = null
  filters.change_type = ""
  page.value = 1
  void load()
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <PageHeader title="库存流水" subtitle="按商品/库位/类型追溯库存变动" />
    <el-card>
      <SearchBar :model-value="filters" @search="page = 1; load()" @reset="reset">
        <el-form-item label="商品">
          <EntitySelect v-model="filters.item_id" :api="searchItems" placeholder="搜索商品" width="220px" />
        </el-form-item>
        <el-form-item label="库位">
          <EntitySelect v-model="filters.location_id" :api="searchLocations" placeholder="搜索库位" width="220px" />
        </el-form-item>
        <el-form-item label="类型">
          <el-select v-model="filters.change_type" clearable style="width: 140px">
            <el-option v-for="o in changeTypeOptions" :key="o.value" :value="o.value" :label="o.label" />
          </el-select>
        </el-form-item>
      </SearchBar>

      <DataTable
        :columns="[
          { prop: 'created_at', label: '时间' },
          { prop: 'item_name', label: '商品' },
          { prop: 'location_name', label: '库位' },
          { prop: 'change_type', label: '类型' },
          { prop: 'quantity', label: '数量' },
          { prop: 'ref_type', label: '关联类型' },
          { prop: 'ref_id', label: '关联 ID' },
        ]"
        :data="rows"
        :total="total"
        :page="page"
        :page-size="pageSize"
        :loading="loading"
        @page-change="page = $event; load()"
        @page-size-change="pageSize = $event; page = 1; load()"
      >
        <template #cell-change_type="{ row }">
          <el-tag :type="changeTypeTag((row as InventoryLog).change_type)">{{ changeTypeLabel((row as InventoryLog).change_type) }}</el-tag>
        </template>
        <template #cell-quantity="{ row }">
          <span :style="{ color: (row as InventoryLog).change_type === 'inbound' ? 'var(--el-color-success)' : 'var(--el-color-warning)' }">
            {{ signedQuantity(row as InventoryLog) }}
          </span>
        </template>
      </DataTable>
    </el-card>
  </section>
</template>