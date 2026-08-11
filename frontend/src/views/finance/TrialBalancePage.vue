<script setup lang="ts">
import { ref, onMounted, computed } from "vue"
import { ElMessage } from "element-plus"
import { get } from "@/api/client"
import type { TrialBalanceRow } from "@/types"

const rows = ref<readonly TrialBalanceRow[]>([])
const loading = ref(false)

const totalDebit = computed(() => rows.value.reduce((s, r) => s + Number(r.total_debit), 0))
const totalCredit = computed(() => rows.value.reduce((s, r) => s + Number(r.total_credit), 0))
const balanced = computed(() => Math.round((totalDebit.value - totalCredit.value) * 10000) / 10000 === 0)

async function load(): Promise<void> {
  loading.value = true
  try {
    const result = await get<readonly TrialBalanceRow[]>("/trial-balance")
    rows.value = result
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally { loading.value = false }
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <div class="heading"><h2>试算平衡表</h2><el-button @click="load">刷新</el-button></div>
    <el-alert v-if="!balanced && rows.length > 0" title="借方总额与贷方总额不平衡" type="warning" :closable="false" show-icon />
    <el-alert v-if="balanced && rows.length > 0" title="借贷平衡 ✓" type="success" :closable="false" show-icon />
    <el-card>
      <el-table :data="rows" v-loading="loading" border show-summary :summary-method="() => [
        '合计', '', '', totalDebit.toFixed(2), totalCredit.toFixed(2), '',
      ]">
        <el-table-column prop="account_id" label="科目 ID" width="80" />
        <el-table-column prop="account_code" label="科目代码" width="120" />
        <el-table-column prop="account_name" label="科目名称" />
        <el-table-column prop="total_debit" label="借方合计" align="right" />
        <el-table-column prop="total_credit" label="贷方合计" align="right" />
        <el-table-column prop="balance" label="余额" align="right" />
      </el-table>
    </el-card>
  </section>
</template>

<style scoped>
.heading { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.heading h2 { margin: 0; }
.el-alert { margin-bottom: 12px; }
</style>
