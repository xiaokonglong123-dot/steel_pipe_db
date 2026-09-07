<script setup lang="ts">
import { computed, onMounted, ref } from "vue"
import { ElMessage } from "element-plus"
import { MoneyText, PageHeader } from "@/components"
import { trialBalance } from "@/api/finance"
import { add } from "@/utils/money"
import type { TrialBalanceRow } from "@/types/finance"

const rows = ref<TrialBalanceRow[]>([])
const loading = ref(false)

const totalDebit = computed(() => add(...rows.value.map((r) => r.total_debit)))
const totalCredit = computed(() => add(...rows.value.map((r) => r.total_credit)))

async function load(): Promise<void> {
  loading.value = true
  try {
    rows.value = [...(await trialBalance())]
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <PageHeader title="试算平衡" subtitle="各科目借贷汇总，借贷总额应相等" />

    <el-card>
      <el-table :data="rows" v-loading="loading" border stripe show-summary :summary-method="() => []">
        <el-table-column prop="account_code" label="科目编码" width="160" />
        <el-table-column prop="account_name" label="科目名称" min-width="200" />
        <el-table-column label="借方合计" align="right" width="180">
          <template #default="{ row }"><MoneyText :value="(row as TrialBalanceRow).total_debit" /></template>
        </el-table-column>
        <el-table-column label="贷方合计" align="right" width="180">
          <template #default="{ row }"><MoneyText :value="(row as TrialBalanceRow).total_credit" /></template>
        </el-table-column>
        <el-table-column label="余额" align="right" width="180">
          <template #default="{ row }"><MoneyText :value="(row as TrialBalanceRow).balance" strong /></template>
        </el-table-column>
      </el-table>

      <div class="totals">
        <span>借方总额 <MoneyText :value="totalDebit" strong /></span>
        <span>贷方总额 <MoneyText :value="totalCredit" strong /></span>
        <el-tag v-if="totalDebit === totalCredit" type="success" size="small">平衡</el-tag>
      </div>
    </el-card>
  </section>
</template>

<style scoped>
.totals { margin-top: 16px; display: flex; gap: 24px; align-items: center; }
</style>