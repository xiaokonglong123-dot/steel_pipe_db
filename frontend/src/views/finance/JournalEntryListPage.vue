<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue"
import { ElMessage } from "element-plus"
import { DataTable, EntitySelect, MoneyText, PageHeader } from "@/components"
import { listJournalEntries, createJournalEntry, postJournalEntry } from "@/api/finance"
import { searchAccounts } from "@/api/selects"
import { useHasPermission } from "@/composables/useHasPermission"
import { add, eq } from "@/utils/money"
import type { JournalEntry, JournalEntryPayload } from "@/types/finance"

const can = useHasPermission()

const rows = ref<JournalEntry[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const loading = ref(false)

interface LineInput { account_id: number | null; debit: string; credit: string }
const dialog = ref(false)
const saving = ref(false)
const form = reactive<{ entry_date: string; description: string; lines: LineInput[] }>({
  entry_date: new Date().toISOString().slice(0, 10),
  description: "",
  lines: [
    { account_id: null, debit: "0", credit: "0" },
    { account_id: null, debit: "0", credit: "0" },
  ],
})

const totalDebit = computed(() => add(...form.lines.map((l) => l.debit || "0")))
const totalCredit = computed(() => add(...form.lines.map((l) => l.credit || "0")))
const balanced = computed(() => eq(totalDebit.value, totalCredit.value))

function statusLabel(s: JournalEntry["status"]): string {
  if (s === "posted") return "已过账"
  if (s === "voided") return "已作废"
  return "草稿"
}
function statusTag(s: JournalEntry["status"]): "success" | "info" | "warning" {
  if (s === "posted") return "success"
  if (s === "voided") return "info"
  return "warning"
}

async function load(): Promise<void> {
  loading.value = true
  try {
    const r = await listJournalEntries()
    rows.value = [...r.items]
    total.value = r.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

function addLine(): void {
  form.lines.push({ account_id: null, debit: "0", credit: "0" })
}
function removeLine(idx: number): void {
  if (form.lines.length <= 2) {
    ElMessage.warning("至少需要两行（一借一贷）")
    return
  }
  form.lines.splice(idx, 1)
}

async function create(): Promise<void> {
  for (const [i, l] of form.lines.entries()) {
    if (l.account_id == null) {
      ElMessage.warning(`第 ${i + 1} 行需选择科目`)
      return
    }
  }
  if (!balanced.value) {
    ElMessage.error(`借贷不平衡（借 ${totalDebit.value} / 贷 ${totalCredit.value}）`)
    return
  }
  saving.value = true
  try {
    const payload: JournalEntryPayload = {
      entry_date: form.entry_date,
      ...(form.description ? { description: form.description } : {}),
      lines: form.lines.map((l) => ({
        account_id: l.account_id as number,
        debit: l.debit || "0",
        credit: l.credit || "0",
      })),
    }
    await createJournalEntry(payload)
    dialog.value = false
    ElMessage.success("日记账已创建")
    void load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "保存失败")
  } finally {
    saving.value = false
  }
}

async function post(row: JournalEntry): Promise<void> {
  try {
    await postJournalEntry(row.id)
    ElMessage.success("已过账")
    void load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "过账失败")
  }
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <PageHeader title="日记账" subtitle="多行借贷记账，借贷平衡后过账">
      <el-button v-if="can('finance.write')" type="primary" @click="dialog = true">新建日记账</el-button>
    </PageHeader>

    <el-card>
      <DataTable
        :columns="[
          { prop: 'entry_no', label: '凭证号' },
          { prop: 'entry_date', label: '日期' },
          { prop: 'description', label: '摘要' },
          { prop: 'status', label: '状态' },
        ]"
        :data="rows"
        :total="total"
        :page="page"
        :page-size="pageSize"
        :loading="loading"
        @page-change="page = $event; load()"
        @page-size-change="pageSize = $event; page = 1; load()"
      >
        <template #cell-status="{ row }">
          <el-tag :type="statusTag((row as JournalEntry).status)">{{ statusLabel((row as JournalEntry).status) }}</el-tag>
        </template>
        <template #actions="{ row }">
          <el-button
            v-if="can('finance.write') && (row as JournalEntry).status === 'draft'"
            link
            type="success"
            @click="post(row as JournalEntry)"
          >
            过账
          </el-button>
        </template>
      </DataTable>
    </el-card>

    <el-dialog v-model="dialog" title="新建日记账" width="820px" destroy-on-close>
      <el-form label-width="80px">
        <el-form-item label="日期">
          <el-date-picker v-model="form.entry_date" type="date" value-format="YYYY-MM-DD" />
        </el-form-item>
        <el-form-item label="摘要"><el-input v-model="form.description" /></el-form-item>
      </el-form>

      <el-table :data="form.lines" border size="small">
        <el-table-column type="index" label="#" width="46" />
        <el-table-column label="科目" min-width="260">
          <template #default="{ row }">
            <EntitySelect v-model="row.account_id" :api="searchAccounts" placeholder="选择科目" />
          </template>
        </el-table-column>
        <el-table-column label="借方" width="180">
          <template #default="{ row }">
            <el-input v-model="row.debit" placeholder="0" />
          </template>
        </el-table-column>
        <el-table-column label="贷方" width="180">
          <template #default="{ row }">
            <el-input v-model="row.credit" placeholder="0" />
          </template>
        </el-table-column>
        <el-table-column label="操作" width="70" align="center">
          <template #default="{ $index }">
            <el-button link type="danger" @click="removeLine($index)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="summary-line">
        <el-button size="small" @click="addLine">+ 添加行</el-button>
        <span class="balance" :class="{ unbalanced: !balanced }">
          借 <MoneyText :value="totalDebit" strong /> ｜ 贷 <MoneyText :value="totalCredit" strong />
          <el-tag v-if="balanced" type="success" size="small" style="margin-left: 8px">平衡</el-tag>
          <el-tag v-else type="danger" size="small" style="margin-left: 8px">不平衡</el-tag>
        </span>
      </div>

      <template #footer>
        <el-button @click="dialog = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="create">保存</el-button>
      </template>
    </el-dialog>
  </section>
</template>

<style scoped>
.summary-line { margin-top: 12px; display: flex; justify-content: space-between; align-items: center; }
.balance { font-variant-numeric: tabular-nums; }
</style>