<script setup lang="ts">
import { onMounted, reactive, ref } from "vue"
import { ElMessage, ElMessageBox } from "element-plus"
import PageHeader from "../common/PageHeader.vue"
import { DataTable, SearchBar } from "@/components"
import { useHasPermission } from "@/composables/useHasPermission"
import type { Page } from "@/types"

export interface MasterColumn {
  prop: string
  label: string
  required?: boolean
  formType?: "text" | "textarea"
  width?: number
}

type Row = Record<string, unknown>

type FetchQuery = { page: number; page_size: number } & Record<string, string>

const props = withDefaults(
  defineProps<{
    title: string
    subtitle?: string
    columns: readonly MasterColumn[]
    searchFields?: readonly string[]
    fetchPage: (q: { page: number; page_size: number } & Record<string, string>) => Promise<Page<Row>>
    createFn?: (row: Row) => Promise<unknown>
    updateFn?: (id: number, row: Row) => Promise<unknown>
    deleteFn?: (id: number) => Promise<unknown>
    writePermission?: string
    showAdd?: boolean
  }>(),
  { searchFields: () => [], subtitle: "", writePermission: "", showAdd: true },
)

const can = useHasPermission()
const canWrite = () => props.writePermission === "" || can.value(props.writePermission)

const rows = ref<Row[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const loading = ref(false)
const dialog = ref(false)
const editingId = ref<number | null>(null)
const saving = ref(false)
const filters = reactive<Record<string, string>>({})
const form = reactive<Record<string, unknown>>({})

function queryStr(): string {
  const params = new URLSearchParams({ page: String(page.value), page_size: String(pageSize.value) })
  for (const key of props.searchFields) {
    const v = filters[key]
    if (v) params.set(key, v)
  }
  return params.toString()
}

async function load(): Promise<void> {
  loading.value = true
  try {
    const result = await props.fetchPage({
      page: page.value,
      page_size: pageSize.value,
      ...filters,
    } as FetchQuery)
    rows.value = [...result.items]
    total.value = result.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

function doSearch(): void {
  page.value = 1
  void load()
}

function reset(): void {
  for (const key of props.searchFields) filters[key] = ""
  page.value = 1
  void load()
}

function openCreate(): void {
  if (!canWrite()) return
  for (const key of Object.keys(form)) delete form[key]
  editingId.value = null
  dialog.value = true
}

function openEdit(row: Row): void {
  if (!canWrite()) return
  const id = Number(row["id"])
  if (!Number.isFinite(id)) return
  editingId.value = id
  for (const col of props.columns) form[col.prop] = row[col.prop] ?? ""
  dialog.value = true
}

async function save(): Promise<void> {
  if (!canWrite()) return
  if (!validate()) return
  saving.value = true
  try {
    const payload: Row = {}
    for (const col of props.columns) {
      const v = form[col.prop]
      if (v !== undefined && v !== "") payload[col.prop] = v
    }
    if (editingId.value !== null && props.updateFn) {
      await props.updateFn(editingId.value, payload)
    } else if (props.createFn) {
      await props.createFn(payload)
    }
    dialog.value = false
    ElMessage.success(editingId.value !== null ? "保存成功" : "创建成功")
    void load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "保存失败")
  } finally {
    saving.value = false
  }
}

function validate(): boolean {
  for (const col of props.columns) {
    if (col.required && (form[col.prop] === undefined || String(form[col.prop]).trim() === "")) {
      ElMessage.warning(`请填写${col.label}`)
      return false
    }
  }
  return true
}

async function remove(row: Row): Promise<void> {
  if (!canWrite() || !props.deleteFn) return
  const id = Number(row["id"])
  if (!Number.isFinite(id)) return
  await ElMessageBox.confirm("确定删除这条记录吗？", "提示", { type: "warning" })
  await props.deleteFn(id)
  ElMessage.success("删除成功")
  void load()
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <PageHeader :title="title" :subtitle="subtitle">
      <el-button v-if="canWrite() && showAdd && createFn" type="primary" @click="openCreate">新建</el-button>
    </PageHeader>

    <el-card>
      <SearchBar :model-value="filters" @search="doSearch" @reset="reset">
        <el-form-item v-for="field in searchFields" :key="field" :label="columns.find((c) => c.prop === field)?.label ?? field">
          <el-input v-model="filters[field]" clearable placeholder="搜索" />
        </el-form-item>
      </SearchBar>
      <DataTable
        :columns="columns.map((c) => ({ prop: c.prop, label: c.label, width: c.width }))"
        :data="rows"
        :total="total"
        :page="page"
        :page-size="pageSize"
        :loading="loading"
        @page-change="page = $event; load()"
        @page-size-change="pageSize = $event; page = 1; load()"
      >
        <template #actions="{ row }">
          <slot name="actions" :row="row" />
          <el-button v-if="canWrite() && updateFn" link type="primary" @click="openEdit(row as Row)">编辑</el-button>
          <el-button v-if="canWrite() && deleteFn" link type="danger" @click="remove(row as Row)">删除</el-button>
        </template>
      </DataTable>
    </el-card>

    <el-dialog v-model="dialog" :title="editingId !== null ? '编辑' : '新建'" width="520px" destroy-on-close>
      <el-form label-width="100px">
        <el-form-item v-for="col in columns" :key="col.prop" :label="col.label" :required="col.required">
          <el-input v-if="col.formType !== 'textarea'" v-model="form[col.prop] as string" />
          <el-input v-else v-model="form[col.prop] as string" type="textarea" :rows="2" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialog = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="save">保存</el-button>
      </template>
    </el-dialog>
  </section>
</template>