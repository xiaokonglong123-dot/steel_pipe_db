<script setup lang="ts">
import { reactive, ref, onMounted } from "vue"
import { ElMessage, ElMessageBox } from "element-plus"
import { del, get, post, put } from "@/api/client"
import { DataTable, SearchBar } from "@/components"
import type { FormRow } from "@/types"

type Field = { readonly prop: string; readonly label: string; readonly required?: boolean }
type Props = { readonly title: string; readonly endpoint: string; readonly columns: readonly Field[]; readonly searchFields?: readonly Field[]; readonly permission: string }
const props = defineProps<Props>()
const rows = ref<readonly FormRow[]>([]); const total = ref(0); const page = ref(1); const pageSize = ref(20); const loading = ref(false); const dialog = ref(false); const editingId = ref("")
const filters = reactive<Record<string, string>>({}); const form = reactive<FormRow>({})
const columns = props.columns.map((field) => ({ prop: field.prop, label: field.label }))
function query(): string { const params = new URLSearchParams({ page: String(page.value), page_size: String(pageSize.value) }); for (const [key, value] of Object.entries(filters)) if (value) params.set(key, value); return `?${params.toString()}` }
async function load(): Promise<void> { loading.value = true; try { const result = await get<{ readonly items: readonly FormRow[]; readonly total: number }>(`${props.endpoint}${query()}`); rows.value = result.items; total.value = result.total } catch (error) { ElMessage.error(error instanceof Error ? error.message : "加载失败") } finally { loading.value = false } }
function reset(): void { for (const key of Object.keys(filters)) filters[key] = ""; page.value = 1; void load() }
function openCreate(): void { editingId.value = ""; for (const key of Object.keys(form)) delete form[key]; dialog.value = true }
function openEdit(row: FormRow): void { const id = row["id"]; if (typeof id !== "string") return; editingId.value = id; for (const field of props.columns) form[field.prop] = row[field.prop]; dialog.value = true }
async function save(): Promise<void> { try { if (editingId.value) await put(`${props.endpoint}/${editingId.value}`, form); else await post(props.endpoint, form); dialog.value = false; ElMessage.success("保存成功"); await load() } catch (error) { ElMessage.error(error instanceof Error ? error.message : "保存失败") } }
async function remove(row: FormRow): Promise<void> { const id = row["id"]; if (typeof id !== "string") return; await ElMessageBox.confirm("确定删除这条记录吗？", "提示", { type: "warning" }); await del(`${props.endpoint}/${id}`); ElMessage.success("删除成功"); await load() }
onMounted(() => void load())
</script>
<template>
  <section class="page"><div class="page-heading"><div><h2>{{ title }}</h2><p>维护基础业务数据</p></div><el-button type="primary" @click="openCreate">新建</el-button></div>
    <el-card><SearchBar :model-value="filters" @search="load" @reset="reset"><el-form-item v-for="field in searchFields ?? []" :key="field.prop" :label="field.label"><el-input v-model="filters[field.prop]" clearable /></el-form-item></SearchBar>
      <DataTable :columns="columns" :data="rows" :total="total" :page="page" :page-size="pageSize" :loading="loading" @page-change="page = $event; load()" @page-size-change="pageSize = $event; page = 1; load()"><template #actions="{ row }"><el-button link type="primary" @click="openEdit(row as FormRow)">编辑</el-button><el-button link type="danger" @click="remove(row as FormRow)">删除</el-button></template></DataTable>
    </el-card>
    <el-dialog v-model="dialog" :title="editingId ? '编辑' : '新建'" width="520px"><el-form :model="form" label-width="100px"><el-form-item v-for="field in columns" :key="field.prop" :label="field.label" :required="props.columns.find((item) => item.prop === field.prop)?.required"><el-input v-model="form[field.prop]" /></el-form-item></el-form><template #footer><el-button @click="dialog = false">取消</el-button><el-button type="primary" @click="save">保存</el-button></template></el-dialog>
  </section>
</template>
<style scoped>.page-heading { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }.page-heading h2 { margin: 0 0 8px; }.page-heading p { margin: 0; color: var(--muted); }</style>
