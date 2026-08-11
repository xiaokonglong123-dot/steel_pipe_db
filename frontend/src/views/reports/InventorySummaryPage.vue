<script setup lang="ts">
import { ref, onMounted, nextTick, onBeforeUnmount } from "vue"
import { ElMessage } from "element-plus"
import * as echarts from "echarts"
import { get } from "@/api/client"
import type { InventorySummaryRow } from "@/types"

const rows = ref<readonly InventorySummaryRow[]>([])
const loading = ref(false)
const chartEl = ref<HTMLDivElement | null>(null)
let chart: echarts.ECharts | null = null

async function load(): Promise<void> {
  loading.value = true
  try {
    rows.value = await get<readonly InventorySummaryRow[]>("/reports/inventory-summary")
    await nextTick()
    renderChart()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally { loading.value = false }
}

function renderChart(): void {
  if (!chart || !chartEl.value) return
  const top = [...rows.value].sort((a, b) => b.total_qty - a.total_qty).slice(0, 10)
  chart.setOption({
    tooltip: { trigger: "axis" },
    grid: { left: 100, right: 30, top: 30, bottom: 40 },
    xAxis: { type: "value", name: "总数量" },
    yAxis: { type: "category", data: top.map((r) => r.sku), inverse: false },
    series: [{ name: "库存数量", type: "bar", data: top.map((r) => r.total_qty), itemStyle: { color: "#409EFF" } }],
    title: { text: "Top 10 库存商品", left: "center", textStyle: { fontSize: 14 } },
  })
}

function exportCsv(): void {
  downloadCsv("/reports/inventory-summary?format=csv", "inventory_summary.csv")
}

function downloadCsv(path: string, fallbackName: string): void {
  const auth = JSON.parse(localStorage.getItem("auth") ?? "{}")
  const token = auth?.auth_token
  fetch(`/api/v1${path}`, { headers: token ? { Authorization: `Bearer ${token}` } : {} })
    .then((r) => {
      if (!r.ok) throw new Error("导出失败")
      const disp = r.headers.get("Content-Disposition") ?? ""
      const m = disp.match(/filename=([^;]+)/)
      const filename = (m ? m[1] : null) ?? fallbackName
      return r.blob().then((b) => ({ filename, blob: b }))
    })
    .then(({ filename, blob }) => {
      const url = URL.createObjectURL(blob)
      const a = document.createElement("a")
      a.href = url; a.download = filename
      document.body.appendChild(a); a.click(); a.remove()
      URL.revokeObjectURL(url)
    })
    .catch((e) => ElMessage.error(e instanceof Error ? e.message : "下载失败"))
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
    <div class="heading">
      <h2>库存汇总报表</h2>
      <div>
        <el-button @click="load">刷新</el-button>
        <el-button type="primary" @click="exportCsv">导出 CSV</el-button>
      </div>
    </div>
    <el-card class="chart-card">
      <div ref="chartEl" class="chart" />
    </el-card>
    <el-card>
      <el-table :data="rows" v-loading="loading" border>
        <el-table-column prop="item_id" label="商品 ID" width="80" />
        <el-table-column prop="sku" label="SKU" />
        <el-table-column prop="name" label="名称" />
        <el-table-column prop="category" label="分类" />
        <el-table-column prop="total_qty" label="总数量" align="right" />
        <el-table-column prop="location_count" label="库位数" width="80" align="right" />
      </el-table>
    </el-card>
  </section>
</template>

<style scoped>
.heading { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.heading h2 { margin: 0; }
.chart-card { margin-bottom: 16px; }
.chart { height: 320px; }
</style>
