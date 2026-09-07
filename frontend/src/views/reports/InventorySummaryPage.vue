<script setup lang="ts">
import { ref, onMounted, nextTick, onBeforeUnmount } from "vue"
import { ElMessage } from "element-plus"
import { echarts } from "@/utils/echarts"
import { MoneyText, PageHeader } from "@/components"
import { inventorySummary, downloadCsv } from "@/api/reports"
import type { InventorySummaryRow } from "@/types/reports"

const rows = ref<InventorySummaryRow[]>([])
const loading = ref(false)
const chartEl = ref<HTMLDivElement | null>(null)
let chart: echarts.ECharts | null = null

async function load(): Promise<void> {
  loading.value = true
  try {
    rows.value = [...(await inventorySummary())]
    await nextTick()
    renderChart()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

function renderChart(): void {
  if (!chart || !chartEl.value) return
  const top = [...rows.value]
    .sort((a, b) => Number(b.total_qty) - Number(a.total_qty))
    .slice(0, 10)
  chart.setOption({
    title: { text: "Top 10 库存商品", left: "center", textStyle: { fontSize: 14 } },
    tooltip: { trigger: "axis" },
    grid: { left: 100, right: 30, top: 40, bottom: 40 },
    xAxis: { type: "value", name: "总数量" },
    yAxis: { type: "category", data: top.map((r) => r.sku), inverse: true },
    series: [{ name: "库存数量", type: "bar", data: top.map((r) => Number(r.total_qty)), itemStyle: { color: "#409EFF" } }],
  })
}

async function exportCsv(): Promise<void> {
  try {
    await downloadCsv("/reports/inventory-summary?format=csv", "inventory_summary.csv")
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "下载失败")
  }
}

onMounted(() => {
  if (chartEl.value) chart = echarts.init(chartEl.value)
  void load()
})

onBeforeUnmount(() => {
  chart?.dispose()
  chart = null
})
</script>

<template>
  <section class="page">
    <PageHeader title="库存汇总" subtitle="各商品库存总量（跨库位汇总）">
      <el-button type="primary" @click="exportCsv">导出 CSV</el-button>
    </PageHeader>

    <el-card class="chart-card">
      <div ref="chartEl" class="chart" />
    </el-card>

    <el-card>
      <el-table :data="rows" v-loading="loading" border stripe>
        <el-table-column prop="sku" label="SKU" width="160" />
        <el-table-column prop="name" label="名称" min-width="180" />
        <el-table-column prop="category" label="分类" width="140" />
        <el-table-column label="总数量" align="right" width="140">
          <template #default="{ row }"><MoneyText :value="(row as InventorySummaryRow).total_qty" kind="qty" strong /></template>
        </el-table-column>
        <el-table-column prop="location_count" label="库位数" width="100" />
      </el-table>
    </el-card>
  </section>
</template>

<style scoped>
.chart-card { margin-bottom: 16px; }
.chart { height: 320px; }
</style>