<script setup lang="ts">
import { ref, reactive, onMounted, computed } from "vue"
import { ElMessage } from "element-plus"
import { get, post } from "@/api/client"
import type { JournalEntry, ApiPage, Account } from "@/types"

const rows = ref<readonly JournalEntry[]>([])
const total = ref(0); const page = ref(1); const pageSize = ref(20); const loading = ref(false)
const dialog = ref(false)
const accounts = ref<readonly Account[]>([])

interface JournalLineInput { account_id: number; debit: string; credit: string; memo: string | null }

const form = reactive<{ entry_date: string; memo: string | null; lines: JournalLineInput[] }>({
  entry_date: new Date().toISOString().slice(0, 10),
  memo: null,
  lines: [
    { account_id: 0, debit: "0", credit: "0", memo: null },
    { account_id: 0, debit: "0", credit: "0", memo: null },
  ],
})

const totalDebit = computed(() => form.lines.reduce((s, l) => s + Number(l.debit || 0), 0))
const totalCredit = computed(() => form.lines.reduce((s, l) => s + Number(l.credit || 0), 0))
const balanced = computed(() => Math.round((totalDebit.value - totalCredit.value) * 10000) / 10000 === 0)

async function load(): Promise<void> {
  loading.value = true
  try {
    const result = await get<ApiPage<JournalEntry>>(`/journal-entries?page=${page.value}&page_size=${pageSize.value}`)
    rows.value = result.items; total.value = result.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally { loading.value = false }
}

async function loadAccounts(): Promise<void> {
  try {
    accounts.value = await get<readonly Account[]>("/accounts")
  } catch {
    ElMessage.warning("加载科目失败")
  }
}

function addLine(): void {
  form.lines.push({ account_id: 0, debit: "0", credit: "0", memo: null })
}
function removeLine(idx: number): void {
  if (form.lines.length <= 2) { ElMessage.warning("至少需要两行（借贷）"); return }
  form.lines.splice(idx, 1)
}

function statusLabel(s: JournalEntry["status"]): string {
  return { draft: "草稿", posted: "已过账", voided: "已作废" }[s] ?? s
}

async function create(): Promise<void> {
  if (!balanced.value) { ElMessage.error("借贷必须平衡"); return }
  if (form.lines.some((l) => !l.account_id)) { ElMessage.error("每行需选科目"); return }
  try {
    await post("/journal-entries", form)
    dialog.value = false
    ElMessage.success("创建成功")
    await load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "保存失败")
  }
}

onMounted(() => {
  void load()
  void loadAccounts()
})
</script>

<template>
  <section class="page">
    <div class="heading"><h2>日记账</h2><el-button type="primary" @click="dialog = true">新建凭证</el-button></div>
    <el-card>
      <el-table :data="rows" v-loading="loading" border>
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="entry_no" label="凭证号" width="140" />
        <el-table-column prop="entry_date" label="日期" width="120" />
        <el-table-column prop="memo" label="摘要" />
        <el-table-column label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="(row as JournalEntry).status === 'posted' ? 'success' : 'info'">
              {{ statusLabel((row as JournalEntry).status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="total_debit" label="借方合计" align="right" />
        <el-table-column prop="total_credit" label="贷方合计" align="right" />
      </el-table>
      <el-pagination :current-page="page" :page-size="pageSize" :total="total" layout="total, prev, pager, next" @current-change="page = $event" />
    </el-card>
    <el-dialog v-model="dialog" title="新建凭证" width="900px">
      <el-form :model="form" label-width="80px">
        <el-form-item label="日期"><el-date-picker v-model="form.entry_date" type="date" value-format="YYYY-MM-DD" /></el-form-item>
        <el-form-item label="摘要"><el-input v-model="form.memo" /></el-form-item>
        <el-divider content-position="left">会计分录</el-divider>
        <el-table :data="form.lines" border size="small">
          <el-table-column label="科目" min-width="200">
            <template #default="{ row }">
              <el-select v-model="(row as JournalLineInput).account_id" placeholder="选择科目" filterable>
                <el-option v-for="a in accounts" :key="a.id" :value="a.id" :label="`${a.code} - ${a.name}`" />
              </el-select>
            </template>
          </el-table-column>
          <el-table-column label="借方" width="140">
            <template #default="{ row }">
              <el-input v-model="(row as JournalLineInput).debit" type="number" min="0" />
            </template>
          </el-table-column>
          <el-table-column label="贷方" width="140">
            <template #default="{ row }">
              <el-input v-model="(row as JournalLineInput).credit" type="number" min="0" />
            </template>
          </el-table-column>
          <el-table-column label="摘要">
            <template #default="{ row }">
              <el-input v-model="(row as JournalLineInput).memo" />
            </template>
          </el-table-column>
          <el-table-column label="操作" width="80">
            <template #default="{ $index }">
              <el-button link type="danger" @click="removeLine($index)">删除</el-button>
            </template>
          </el-table-column>
        </el-table>
        <div class="totals">
          <el-button @click="addLine">添加行</el-button>
          <span class="balance" :class="{ ok: balanced, bad: !balanced }">
            借方合计：{{ totalDebit.toFixed(2) }}　贷方合计：{{ totalCredit.toFixed(2) }}　{{ balanced ? "✓ 平衡" : "✗ 不平衡" }}
          </span>
        </div>
      </el-form>
      <template #footer><el-button @click="dialog = false">取消</el-button><el-button type="primary" @click="create">保存</el-button></template>
    </el-dialog>
  </section>
</template>

<style scoped>
.heading { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.heading h2 { margin: 0; }
.totals { display: flex; justify-content: space-between; align-items: center; padding: 12px 0; }
.balance.ok { color: var(--el-color-success); font-weight: bold; }
.balance.bad { color: var(--el-color-danger); font-weight: bold; }
</style>
