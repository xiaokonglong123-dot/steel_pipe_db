<script setup lang="ts">
import { ref, onMounted, nextTick, onBeforeUnmount } from "vue"
import { ElMessage } from "element-plus"
import { echarts } from "@/utils/echarts"
import { MoneyText, PageHeader } from "@/components"
import { salesTrend, downloadCsv } from "@/api/reports"
import type { SalesTrendRow } from "@/types/reports"

const rows = ref<SalesTrendRow[]>([])
const loading = ref(false)
const chartEl = ref<HTMLDivElement | null>(null)
let chart: echarts.ECharts | null = null

async function load(): Promise<void> {
  loading.value = true
  try {
    rows.value = [...(await salesTrend(12))]
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
  chart.setOption({
    title: { text: "销售趋势（近 12 个月）", left: "center", textStyle: { fontSize: 14 } },
    tooltip: { trigger: "axis" },
    grid: { left: 60, right: 60, top: 50, bottom: 30 },
    legend: { data: ["订单数", "总金额"], bottom: 0 },
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

async function exportCsv(): Promise<void> {
  try {
    await downloadCsv("/reports/sales-trend?format=csv", "sales_trend.csv")
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
    <PageHeader title="销售趋势" subtitle="按月订单数与销售金额趋势">
      <el-button type="primary" @click="exportCsv">导出 CSV</el-button>
    </PageHeader>

    <el-card class="chart-card">
      <div ref="chartEl" class="chart" />
    </el-card>

    <el-card>
      <el-table :data="rows" v-loading="loading" border stripe>
        <el-table-column prop="month" label="月份" width="120" />
        <el-table-column prop="order_count" label="订单数" width="120" />
        <el-table-column label="销售额" align="right">
          <template #default="{ row }"><MoneyText :value="(row as SalesTrendRow).total_amount" strong /></template>
        </el-table-column>
      </el-table>
    </el-card>
  </section>
</template>

<style scoped>
.chart-card { margin-bottom: 16px; }
.chart { height: 320px; }
</style>