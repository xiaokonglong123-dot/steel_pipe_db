<script setup lang="ts">
import { onMounted, reactive, ref } from "vue"
import { ElMessage } from "element-plus"
import { DataTable, EntitySelect, PageHeader, SearchBar } from "@/components"
import { listStock } from "@/api/inventory"
import { searchItems, searchLocations, searchWarehouses } from "@/api/selects"
import type { StockRow } from "@/types/inventory"

const rows = ref<StockRow[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const loading = ref(false)
const filters = reactive<{ item_id: number | null; location_id: number | null; warehouse_id: number | null }>({
  item_id: null, location_id: null, warehouse_id: null,
})

async function load(): Promise<void> {
  loading.value = true
  try {
    const r = await listStock({
      page: page.value,
      page_size: pageSize.value,
      ...(filters.item_id != null ? { item_id: filters.item_id } : {}),
      ...(filters.location_id != null ? { location_id: filters.location_id } : {}),
      ...(filters.warehouse_id != null ? { warehouse_id: filters.warehouse_id } : {}),
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
  filters.warehouse_id = null
  page.value = 1
  void load()
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <PageHeader title="库存查询" subtitle="按商品/库位/仓库查看现有量" />
    <el-card>
      <SearchBar :model-value="filters" @search="page = 1; load()" @reset="reset">
        <el-form-item label="商品">
          <EntitySelect v-model="filters.item_id" :api="searchItems" placeholder="搜索商品" width="220px" />
        </el-form-item>
        <el-form-item label="库位">
          <EntitySelect v-model="filters.location_id" :api="searchLocations" placeholder="搜索库位" width="220px" />
        </el-form-item>
        <el-form-item label="仓库">
          <EntitySelect v-model="filters.warehouse_id" :api="searchWarehouses" placeholder="搜索仓库" width="220px" />
        </el-form-item>
      </SearchBar>

      <DataTable
        :columns="[
          { prop: 'item_name', label: '商品' },
          { prop: 'sku', label: 'SKU' },
          { prop: 'location_name', label: '库位' },
          { prop: 'warehouse_name', label: '仓库' },
          { prop: 'quantity', label: '现有量' },
        ]"
        :data="rows"
        :total="total"
        :page="page"
        :page-size="pageSize"
        :loading="loading"
        @page-change="page = $event; load()"
        @page-size-change="pageSize = $event; page = 1; load()"
      >
        <template #cell-quantity="{ row }">
          <strong>{{ (row as StockRow).quantity }}</strong>
        </template>
      </DataTable>
    </el-card>
  </section>
</template>