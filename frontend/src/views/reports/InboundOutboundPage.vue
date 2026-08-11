<script setup lang="ts">
import { reactive, ref, onMounted, nextTick, onBeforeUnmount } from "vue"
import { ElMessage } from "element-plus"
import * as echarts from "echarts"
import { get } from "@/api/client"
import type { InboundOutboundRow } from "@/types"

const rows = ref<readonly InboundOutboundRow[]>([])
const loading = ref(false)
const filters = reactive({ item_id: "", start_date: "", end_date: "" })
const chartEl = ref<HTMLDivElement | null>(null)
let chart: echarts.ECharts | null = null

async function load(): Promise<void> {
  loading.value = true
  try {
    const params = new URLSearchParams()
    if (filters.item_id) params.set("item_id", filters.item_id)
    if (filters.start_date) params.set("start_date", filters.start_date)
    if (filters.end_date) params.set("end_date", filters.end_date)
    const qs = params.toString() ? `?${params.toString()}` : ""
    rows.value = await get<readonly InboundOutboundRow[]>(`/reports/inbound-outbound${qs}`)
    await nextTick()
    renderChart()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally { loading.value = false }
}

function renderChart(): void {
  if (!chart || !chartEl.value) return
  type Day = { date: string; inbound: number; outbound: number }
  const map = new Map<string, Day>()
  for (const r of rows.value) {
    const day = r.created_at.slice(0, 10)
    if (!map.has(day)) map.set(day, { date: day, inbound: 0, outbound: 0 })
    const o = map.get(day)!
    if (r.change_type === "inbound") o.inbound += r.quantity
    else if (r.change_type === "outbound") o.outbound += Math.abs(r.quantity)
  }
  const days = Array.from(map.values()).sort((a, b) => a.date.localeCompare(b.date))
  chart.setOption({
    title: { text: "每日入/出库合计", left: "center", textStyle: { fontSize: 14 } },
    tooltip: { trigger: "axis" },
    legend: { data: ["入库", "出库"], bottom: 0 },
    grid: { left: 50, right: 30, top: 50, bottom: 50 },
    xAxis: { type: "category", data: days.map((d) => d.date) },
    yAxis: { type: "value" },
    series: [
      { name: "入库", type: "line", smooth: true, data: days.map((d) => d.inbound), areaStyle: { opacity: 0.2 } },
      { name: "出库", type: "line", smooth: true, data: days.map((d) => d.outbound), areaStyle: { opacity: 0.2 } },
    ],
  })
}

function changeTypeLabel(t: string): string {
  return { inbound: "入库", outbound: "出库", check_adjust: "盘点调整" }[t] ?? t
}
function changeTypeTag(t: string): "success" | "warning" | "info" {
  if (t === "inbound") return "success"
  if (t === "outbound") return "warning"
  return "info"
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
  const params = new URLSearchParams()
  if (filters.item_id) params.set("item_id", filters.item_id)
  if (filters.start_date) params.set("start_date", filters.start_date)
  if (filters.end_date) params.set("end_date", filters.end_date)
  params.set("format", "csv")
  downloadCsv(`/reports/inbound-outbound?${params.toString()}`, "inbound_outbound.csv")
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
      <h2>出入库明细</h2>
      <div>
        <el-button @click="load">刷新</el-button>
        <el-button type="primary" @click="exportCsv">导出 CSV</el-button>
      </div>
    </div>
    <el-card class="chart-card">
      <div ref="chartEl" class="chart" />
    </el-card>
    <el-card>
      <el-form inline>
        <el-form-item label="商品 ID"><el-input v-model="filters.item_id" clearable style="width: 120px" /></el-form-item>
        <el-form-item label="开始日期"><el-date-picker v-model="filters.start_date" type="date" value-format="YYYY-MM-DD" /></el-form-item>
        <el-form-item label="结束日期"><el-date-picker v-model="filters.end_date" type="date" value-format="YYYY-MM-DD" /></el-form-item>
        <el-form-item><el-button type="primary" @click="load">查询</el-button></el-form-item>
      </el-form>
      <el-table :data="rows" v-loading="loading" border>
        <el-table-column prop="created_at" label="时间" width="180" />
        <el-table-column prop="log_id" label="流水 ID" width="80" />
        <el-table-column label="类型" width="100">
          <template #default="{ row }">
            <el-tag :type="changeTypeTag((row as InboundOutboundRow).change_type)">
              {{ changeTypeLabel((row as InboundOutboundRow).change_type) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="sku" label="SKU" />
        <el-table-column prop="name" label="名称" />
        <el-table-column prop="quantity" label="数量" align="right" />
        <el-table-column prop="location_id" label="库位 ID" width="100" />
        <el-table-column prop="ref_type" label="关联类型" width="100" />
        <el-table-column prop="ref_id" label="关联 ID" width="80" />
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
