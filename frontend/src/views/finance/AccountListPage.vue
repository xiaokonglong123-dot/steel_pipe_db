<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue"
import { ElMessage } from "element-plus"
import { EntitySelect, PageHeader } from "@/components"
import { listAccounts, createAccount } from "@/api/finance"
import { searchAccounts } from "@/api/selects"
import type { Account, AccountPayload } from "@/types/finance"

const rows = ref<Account[]>([])
const loading = ref(false)
const filterType = ref<"" | Account["account_type"]>("")

const dialog = ref(false)
const saving = ref(false)
const form = reactive<{ code: string; name: string; account_type: Account["account_type"]; parent_id: number | null }>({
  code: "", name: "", account_type: "asset", parent_id: null,
})

const ACCOUNT_TYPES: ReadonlyArray<{ value: Account["account_type"]; label: string }> = [
  { value: "asset", label: "资产" },
  { value: "liability", label: "负债" },
  { value: "equity", label: "权益" },
  { value: "income", label: "收入" },
  { value: "expense", label: "费用" },
]

const filtered = computed(() =>
  filterType.value ? rows.value.filter((a) => a.account_type === filterType.value) : rows.value,
)

const accountTypeLabel = (t: Account["account_type"]) =>
  ACCOUNT_TYPES.find((o) => o.value === t)?.label ?? t

async function load(): Promise<void> {
  loading.value = true
  try {
    rows.value = [...(await listAccounts())]
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

function openCreate(): void {
  form.code = ""
  form.name = ""
  form.account_type = "asset"
  form.parent_id = null
  dialog.value = true
}

async function save(): Promise<void> {
  if (!form.code.trim() || !form.name.trim()) {
    ElMessage.warning("科目编码和名称必填")
    return
  }
  saving.value = true
  try {
    const payload: AccountPayload = {
      code: form.code,
      name: form.name,
      account_type: form.account_type,
      ...(form.parent_id != null ? { parent_id: form.parent_id } : {}),
    }
    await createAccount(payload)
    dialog.value = false
    ElMessage.success("科目已创建")
    void load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "保存失败")
  } finally {
    saving.value = false
  }
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <PageHeader title="会计科目" subtitle="科目树主数据（无删除，避免破坏历史分录）">
      <el-button type="primary" @click="openCreate">新建科目</el-button>
    </PageHeader>

    <el-card>
      <el-form inline>
        <el-form-item label="类型">
          <el-select v-model="filterType" clearable style="width: 160px">
            <el-option v-for="o in ACCOUNT_TYPES" :key="o.value" :value="o.value" :label="o.label" />
          </el-select>
        </el-form-item>
      </el-form>

      <el-table :data="filtered" v-loading="loading" border stripe>
        <el-table-column prop="code" label="编码" width="140" />
        <el-table-column prop="name" label="名称" min-width="180" />
        <el-table-column label="类型" width="100">
          <template #default="{ row }">{{ accountTypeLabel((row as Account).account_type) }}</template>
        </el-table-column>
        <el-table-column prop="parent_name" label="上级科目" min-width="160" />
        <el-table-column prop="is_active" label="启用" width="80" />
      </el-table>
    </el-card>

    <el-dialog v-model="dialog" title="新建科目" width="500px" destroy-on-close>
      <el-form label-width="100px">
        <el-form-item label="编码" required><el-input v-model="form.code" /></el-form-item>
        <el-form-item label="名称" required><el-input v-model="form.name" /></el-form-item>
        <el-form-item label="类型" required>
          <el-select v-model="form.account_type" style="width: 100%">
            <el-option v-for="o in ACCOUNT_TYPES" :key="o.value" :value="o.value" :label="o.label" />
          </el-select>
        </el-form-item>
        <el-form-item label="上级科目">
          <EntitySelect v-model="form.parent_id" :api="searchAccounts" placeholder="无（一级科目）" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialog = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="save">保存</el-button>
      </template>
    </el-dialog>
  </section>
</template>