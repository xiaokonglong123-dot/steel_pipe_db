<script setup lang="ts">
import { reactive, ref, onMounted } from "vue"
import { ElMessage } from "element-plus"
import { del, get, post } from "@/api/client"
import type { Account, ApiPage } from "@/types"

const rows = ref<readonly Account[]>([])
const loading = ref(false)
const dialog = ref(false)
const form = reactive<{ code: string; name: string; account_type: Account["account_type"]; parent_id: number | null }>({
  code: "", name: "", account_type: "asset", parent_id: null,
})
const filterType = ref<"asset" | "liability" | "equity" | "income" | "expense" | "">("")

const accountTypeOptions: ReadonlyArray<{ value: Account["account_type"]; label: string }> = [
  { value: "asset", label: "资产" },
  { value: "liability", label: "负债" },
  { value: "equity", label: "权益" },
  { value: "income", label: "收入" },
  { value: "expense", label: "费用" },
]

function accountTypeLabel(t: Account["account_type"]): string {
  return accountTypeOptions.find((opt) => opt.value === t)?.label ?? t
}

async function load(): Promise<void> {
  loading.value = true
  try {
    const qs = filterType.value ? `?account_type=${filterType.value}` : ""
    const result = await get<readonly Account[] | ApiPage<Account>>(`/accounts${qs}`)
    const data = isPage(result) ? result.items : result
    rows.value = data
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

function isPage(r: readonly Account[] | ApiPage<Account>): r is ApiPage<Account> {
  return Array.isArray((r as ApiPage<Account>).items)
}

function openCreate(): void {
  form.code = ""; form.name = ""; form.account_type = "asset"; form.parent_id = null
  dialog.value = true
}

async function save(): Promise<void> {
  if (!form.code || !form.name) { ElMessage.warning("代码和名称必填"); return }
  try {
    await post("/accounts", form)
    dialog.value = false
    ElMessage.success("创建成功")
    await load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "保存失败")
  }
}

async function remove(row: Account): Promise<void> {
  try {
    await del(`/accounts/${row.id}`)
    ElMessage.success("删除成功")
    await load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "删除失败")
  }
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <div class="heading"><h2>会计科目</h2><el-button type="primary" @click="openCreate">新建科目</el-button></div>
    <el-card>
      <el-form inline>
        <el-form-item label="类型">
          <el-select v-model="filterType" clearable placeholder="全部" @change="load">
            <el-option v-for="opt in accountTypeOptions" :key="opt.value" :value="opt.value" :label="opt.label" />
          </el-select>
        </el-form-item>
      </el-form>
      <el-table :data="rows" v-loading="loading" border>
        <el-table-column prop="code" label="代码" width="120" />
        <el-table-column prop="name" label="名称" />
        <el-table-column label="类型" width="100">
          <template #default="{ row }"><el-tag>{{ accountTypeLabel((row as Account).account_type) }}</el-tag></template>
        </el-table-column>
        <el-table-column prop="is_active" label="启用" width="80">
          <template #default="{ row }">{{ (row as Account).is_active === 1 || (row as Account).is_active === true ? "✓" : "—" }}</template>
        </el-table-column>
        <el-table-column label="操作" width="100">
          <template #default="{ row }"><el-button link type="danger" @click="remove(row as Account)">删除</el-button></template>
        </el-table-column>
      </el-table>
    </el-card>
    <el-dialog v-model="dialog" title="新建科目" width="500px">
      <el-form :model="form" label-width="80px">
        <el-form-item label="代码" required><el-input v-model="form.code" /></el-form-item>
        <el-form-item label="名称" required><el-input v-model="form.name" /></el-form-item>
        <el-form-item label="类型" required>
          <el-select v-model="form.account_type">
            <el-option v-for="opt in accountTypeOptions" :key="opt.value" :value="opt.value" :label="opt.label" />
          </el-select>
        </el-form-item>
        <el-form-item label="父科目"><el-input-number v-model="form.parent_id" :min="0" controls-position="right" /></el-form-item>
      </el-form>
      <template #footer><el-button @click="dialog = false">取消</el-button><el-button type="primary" @click="save">保存</el-button></template>
    </el-dialog>
  </section>
</template>

<style scoped>
.page { padding: 0; }
.heading { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.heading h2 { margin: 0; }
</style>
