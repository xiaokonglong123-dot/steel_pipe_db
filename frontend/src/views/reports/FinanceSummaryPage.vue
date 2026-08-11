<script setup lang="ts">
import { ref, onMounted, computed, nextTick, onBeforeUnmount } from "vue"
import { ElMessage } from "element-plus"
import * as echarts from "echarts"
import { get } from "@/api/client"
import type { FinanceSummaryRow } from "@/types"

const rows = ref<readonly FinanceSummaryRow[]>([])
const loading = ref(false)
const chartEl = ref<HTMLDivElement | null>(null)
let chart: echarts.ECharts | null = null

const totalDebit = computed(() => rows.value.reduce((s, r) => s + Number(r.total_debit), 0))
const totalCredit = computed(() => rows.value.reduce((s, r) => s + Number(r.total_credit), 0))

async function load(): Promise<void> {
  loading.value = true
  try {
    rows.value = await get<readonly FinanceSummaryRow[]>("/reports/finance-summary")
    await nextTick()
    renderChart()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally { loading.value = false }
}

function renderChart(): void {
  if (!chart || !chartEl.value) return
  const grouped = new Map<string, { debit: number; credit: number }>()
  for (const r of rows.value) {
    const acc = (grouped.get(r.account_type) ?? { debit: 0, credit: 0 })
    acc.debit += Number(r.total_debit)
    acc.credit += Number(r.total_credit)
    grouped.set(r.account_type, acc)
  }
  const types = Array.from(grouped.keys())
  chart.setOption({
    title: { text: "各科目类型借贷对比", left: "center", textStyle: { fontSize: 14 } },
    tooltip: { trigger: "axis" },
    legend: { data: ["借方合计", "贷方合计"], bottom: 0 },
    grid: { left: 50, right: 30, top: 50, bottom: 50 },
    xAxis: { type: "category", data: types.map(accountTypeLabel) },
    yAxis: { type: "value" },
    series: [
      { name: "借方合计", type: "bar", data: types.map((t) => grouped.get(t)?.debit ?? 0) },
      { name: "贷方合计", type: "bar", data: types.map((t) => grouped.get(t)?.credit ?? 0) },
    ],
  })
}

function accountTypeLabel(t: string): string {
  return { asset: "资产", liability: "负债", equity: "权益", income: "收入", expense: "费用" }[t] ?? t
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
  downloadCsv("/reports/finance-summary?format=csv", "finance_summary.csv")
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
      <h2>财务汇总</h2>
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
        '合计', '', '', '', totalDebit.toFixed(2), totalCredit.toFixed(2),
      ]">
        <el-table-column prop="account_id" label="科目 ID" width="80" />
        <el-table-column prop="account_code" label="科目代码" width="120" />
        <el-table-column prop="account_name" label="科目名称" />
        <el-table-column label="类型" width="100">
          <template #default="{ row }">{{ accountTypeLabel((row as FinanceSummaryRow).account_type) }}</template>
        </el-table-column>
        <el-table-column prop="total_debit" label="借方合计" align="right" />
        <el-table-column prop="total_credit" label="贷方合计" align="right" />
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
