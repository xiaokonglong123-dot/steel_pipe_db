<script setup lang="ts">
import { reactive, ref, onMounted } from "vue"
import { ElMessage } from "element-plus"
import { get } from "@/api/client"
import type { InventoryLog, ApiPage } from "@/types"

const rows = ref<readonly InventoryLog[]>([])
const total = ref(0); const page = ref(1); const pageSize = ref(20); const loading = ref(false)
const filters = reactive<{ item_id: string; change_type: string; start_date: string; end_date: string }>({
  item_id: "", change_type: "", start_date: "", end_date: "",
})

const changeTypeOptions = [
  { value: "inbound", label: "入库" },
  { value: "outbound", label: "出库" },
  { value: "check_adjust", label: "盘点调整" },
]

function changeTypeLabel(t: string): string {
  return changeTypeOptions.find((o) => o.value === t)?.label ?? t
}
function changeTypeTag(t: string): "success" | "warning" | "info" {
  if (t === "inbound") return "success"
  if (t === "outbound") return "warning"
  return "info"
}
function signedQuantity(row: InventoryLog): string {
  return row.change_type === "inbound"
    ? `+${row.quantity}`
    : `${row.quantity < 0 ? "" : "-"}${Math.abs(row.quantity)}`
}

function query(): string {
  const params = new URLSearchParams({ page: String(page.value), page_size: String(pageSize.value) })
  if (filters.item_id) params.set("item_id", filters.item_id)
  if (filters.change_type) params.set("change_type", filters.change_type)
  if (filters.start_date) params.set("start_date", filters.start_date)
  if (filters.end_date) params.set("end_date", filters.end_date)
  return `?${params.toString()}`
}

async function load(): Promise<void> {
  loading.value = true
  try {
    const result = await get<ApiPage<InventoryLog>>(`/inventory-logs${query()}`)
    rows.value = result.items; total.value = result.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally { loading.value = false }
}

function reset(): void {
  filters.item_id = ""; filters.change_type = ""; filters.start_date = ""; filters.end_date = ""
  page.value = 1
  void load()
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <div class="heading"><h2>库存流水</h2></div>
    <el-card>
      <el-form inline>
        <el-form-item label="商品 ID"><el-input v-model="filters.item_id" clearable style="width: 120px" /></el-form-item>
        <el-form-item label="类型">
          <el-select v-model="filters.change_type" clearable style="width: 120px">
            <el-option v-for="opt in changeTypeOptions" :key="opt.value" :value="opt.value" :label="opt.label" />
          </el-select>
        </el-form-item>
        <el-form-item label="开始"><el-date-picker v-model="filters.start_date" type="date" value-format="YYYY-MM-DD" /></el-form-item>
        <el-form-item label="结束"><el-date-picker v-model="filters.end_date" type="date" value-format="YYYY-MM-DD" /></el-form-item>
        <el-form-item><el-button type="primary" @click="load">查询</el-button><el-button @click="reset">重置</el-button></el-form-item>
      </el-form>
      <el-table :data="rows" v-loading="loading" border>
        <el-table-column prop="created_at" label="时间" width="180" />
        <el-table-column prop="item_id" label="商品 ID" width="100" />
        <el-table-column prop="location_id" label="库位 ID" width="100" />
        <el-table-column label="类型" width="120">
          <template #default="{ row }">
            <el-tag :type="changeTypeTag((row as InventoryLog).change_type)">{{ changeTypeLabel((row as InventoryLog).change_type) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="数量" align="right" width="120">
          <template #default="{ row }"><span>{{ signedQuantity(row as InventoryLog) }}</span></template>
        </el-table-column>
        <el-table-column prop="ref_type" label="关联类型" width="120" />
        <el-table-column prop="ref_id" label="关联 ID" width="100" />
      </el-table>
      <el-pagination :current-page="page" :page-size="pageSize" :total="total" layout="total, prev, pager, next" @current-change="page = $event; load()" />
    </el-card>
  </section>
</template>

<style scoped>
.heading { margin-bottom: 16px; }
.heading h2 { margin: 0; }
</style>
