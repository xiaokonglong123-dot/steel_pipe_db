<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, nextTick } from "vue"
import { ElMessage } from "element-plus"
import * as echarts from "echarts"
import { get } from "@/api/client"
import type { SalesTrendRow } from "@/types"

const rows = ref<readonly SalesTrendRow[]>([])
const loading = ref(false)
const chartEl = ref<HTMLDivElement | null>(null)
let chart: echarts.ECharts | null = null

async function load(): Promise<void> {
  loading.value = true
  try {
    rows.value = await get<readonly SalesTrendRow[]>("/reports/sales-trend")
    await nextTick()
    renderChart()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally { loading.value = false }
}

function renderChart(): void {
  if (!chart || !chartEl.value) return
  chart.setOption({
    tooltip: { trigger: "axis" },
    grid: { left: 50, right: 30, top: 40, bottom: 30 },
    legend: { data: ["订单数", "总金额"] },
    xAxis: { type: "category", data: rows.value.map((r) => r.month) },
    yAxis: [
      { type: "value", name: "订单数" },
      { type: "value", name: "金额" },
    ],
    series: [
      { name: "订单数", type: "bar", data: rows.value.map((r) => r.order_count) },
      { name: "总金额", type: "line", smooth: true, yAxisIndex: 1, data: rows.value.map((r) => Number(r.total_amount)) },
    ],
  })
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
function exportCsv(): void {
  downloadCsv("/reports/sales-trend?format=csv", "sales_trend.csv")
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
      <h2>销售趋势</h2>
      <div>
        <el-button @click="load">刷新</el-button>
        <el-button type="primary" @click="exportCsv">导出 CSV</el-button>
      </div>
    </div>
    <el-card class="chart-card">
      <div ref="chartEl" class="chart" />
    </el-card>
    <el-card>
      <el-table :data="rows" v-loading="loading" border show-summary :summary-method="() => [
        '合计', rows.reduce((s, r) => s + r.order_count, 0), rows.reduce((s, r) => s + Number(r.total_amount), 0).toFixed(2),
      ]">
        <el-table-column prop="month" label="月份" width="120" />
        <el-table-column prop="order_count" label="订单数" align="right" />
        <el-table-column label="总金额" align="right">
          <template #default="{ row }">{{ row.total_amount }}</template>
        </el-table-column>
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
