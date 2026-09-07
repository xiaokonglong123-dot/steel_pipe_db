<script setup lang="ts">
import { ref, onMounted, computed, nextTick, onBeforeUnmount } from "vue"
import { ElMessage } from "element-plus"
import { echarts } from "@/utils/echarts"
import { MoneyText, PageHeader } from "@/components"
import { financeSummary, downloadCsv } from "@/api/reports"
import { add } from "@/utils/money"
import type { FinanceSummaryRow } from "@/types/reports"

const rows = ref<FinanceSummaryRow[]>([])
const loading = ref(false)
const chartEl = ref<HTMLDivElement | null>(null)
let chart: echarts.ECharts | null = null

const totalDebit = computed(() => add(...rows.value.map((r) => r.total_debit)))
const totalCredit = computed(() => add(...rows.value.map((r) => r.total_credit)))

async function load(): Promise<void> {
  loading.value = true
  try {
    rows.value = [...(await financeSummary())]
    await nextTick()
    renderChart()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

function accountTypeLabel(t: string): string {
  return { asset: "资产", liability: "负债", equity: "权益", income: "收入", expense: "费用" }[t] ?? t
}

function renderChart(): void {
  if (!chart || !chartEl.value) return
  const grouped = new Map<string, { debit: number; credit: number }>()
  for (const r of rows.value) {
    const acc = grouped.get(r.account_type) ?? { debit: 0, credit: 0 }
    acc.debit += Number(r.total_debit)
    acc.credit += Number(r.total_credit)
    grouped.set(r.account_type, acc)
  }
  const types = Array.from(grouped.keys())
  chart.setOption({
    title: { text: "各科目类型借贷对比", left: "center", textStyle: { fontSize: 14 } },
    tooltip: { trigger: "axis" },
    legend: { data: ["借方合计", "贷方合计"], bottom: 0 },
    grid: { left: 60, right: 30, top: 50, bottom: 50 },
    xAxis: { type: "category", data: types.map(accountTypeLabel) },
    yAxis: { type: "value" },
    series: [
      { name: "借方合计", type: "bar", data: types.map((t) => grouped.get(t)?.debit ?? 0) },
      { name: "贷方合计", type: "bar", data: types.map((t) => grouped.get(t)?.credit ?? 0) },
    ],
  })
}

async function exportCsv(): Promise<void> {
  try {
    await downloadCsv("/reports/finance-summary?format=csv", "finance_summary.csv")
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
    <PageHeader title="财务汇总" subtitle="各科目借贷汇总">
      <el-button type="primary" @click="exportCsv">导出 CSV</el-button>
    </PageHeader>

    <el-card class="chart-card">
      <div ref="chartEl" class="chart" />
    </el-card>

    <el-card>
      <el-table :data="rows" v-loading="loading" border stripe>
        <el-table-column prop="account_code" label="编码" width="140" />
        <el-table-column prop="account_name" label="科目" min-width="180" />
        <el-table-column label="类型" width="100">
          <template #default="{ row }">{{ accountTypeLabel((row as FinanceSummaryRow).account_type) }}</template>
        </el-table-column>
        <el-table-column label="借方合计" align="right" width="160">
          <template #default="{ row }"><MoneyText :value="(row as FinanceSummaryRow).total_debit" /></template>
        </el-table-column>
        <el-table-column label="贷方合计" align="right" width="160">
          <template #default="{ row }"><MoneyText :value="(row as FinanceSummaryRow).total_credit" /></template>
        </el-table-column>
      </el-table>

      <div class="totals">
        <span>借方总额 <MoneyText :value="totalDebit" strong /></span>
        <span>贷方总额 <MoneyText :value="totalCredit" strong /></span>
      </div>
    </el-card>
  </section>
</template>

<style scoped>
.chart-card { margin-bottom: 16px; }
.chart { height: 320px; }
.totals { margin-top: 16px; display: flex; gap: 24px; }
</style>