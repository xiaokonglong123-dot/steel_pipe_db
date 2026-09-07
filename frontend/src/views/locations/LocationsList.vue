<script setup lang="ts">
import { onMounted, reactive, ref } from "vue"
import { ElMessage, ElMessageBox } from "element-plus"
import { DataTable, PageHeader, SearchBar, EntitySelect } from "@/components"
import { listLocations, createLocation, updateLocation, deleteLocation } from "@/api/locations"
import { searchWarehouses } from "@/api/selects"
import { useHasPermission } from "@/composables/useHasPermission"
import type { Location, LocationPayload } from "@/types/locations"

const can = useHasPermission()
const canWrite = () => can.value("stock.write")

const rows = ref<Location[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const loading = ref(false)
const filters = reactive<{ code: string; name: string }>({ code: "", name: "" })

const dialog = ref(false)
const editingId = ref<number | null>(null)
const saving = ref(false)
const form = reactive<{ warehouse_id: number | null; code: string; name: string }>({
  warehouse_id: null, code: "", name: "",
})

const columns = [
  { prop: "warehouse_name", label: "所属仓库" },
  { prop: "code", label: "编码" },
  { prop: "name", label: "库位名称" },
]

async function load(): Promise<void> {
  loading.value = true
  try {
    const r = await listLocations({
      page: page.value,
      page_size: pageSize.value,
      ...(filters.code ? { code: filters.code } : {}),
      ...(filters.name ? { name: filters.name } : {}),
    })
    rows.value = [...r.items]
    total.value = r.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

function reset(): void {
  filters.code = ""
  filters.name = ""
  page.value = 1
  void load()
}

function openCreate(): void {
  editingId.value = null
  form.warehouse_id = null
  form.code = ""
  form.name = ""
  dialog.value = true
}

function openEdit(row: Location): void {
  editingId.value = row.id
  form.warehouse_id = row.warehouse_id
  form.code = row.code
  form.name = row.name
  dialog.value = true
}

async function save(): Promise<void> {
  if (!form.code.trim() || !form.name.trim()) {
    ElMessage.warning("请填写编码和名称")
    return
  }
  saving.value = true
  try {
    const payload: LocationPayload = {
      warehouse_id: form.warehouse_id ?? null,
      code: form.code,
      name: form.name,
    }
    if (editingId.value !== null) await updateLocation(editingId.value, payload)
    else await createLocation(payload)
    dialog.value = false
    ElMessage.success("保存成功")
    void load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "保存失败")
  } finally {
    saving.value = false
  }
}

async function remove(row: Location): Promise<void> {
  await ElMessageBox.confirm("确定删除这个库位吗？", "提示", { type: "warning" })
  await deleteLocation(row.id)
  ElMessage.success("删除成功")
  void load()
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <PageHeader title="库位" subtitle="维护库存库位及其所属仓库">
      <el-button v-if="canWrite()" type="primary" @click="openCreate">新建</el-button>
    </PageHeader>

    <el-card>
      <SearchBar :model-value="filters" @search="page = 1; load()" @reset="reset">
        <el-form-item label="编码"><el-input v-model="filters.code" clearable /></el-form-item>
        <el-form-item label="名称"><el-input v-model="filters.name" clearable /></el-form-item>
      </SearchBar>
      <DataTable
        :columns="columns"
        :data="rows"
        :total="total"
        :page="page"
        :page-size="pageSize"
        :loading="loading"
        @page-change="page = $event; load()"
        @page-size-change="pageSize = $event; page = 1; load()"
      >
        <template #actions="{ row }">
          <el-button v-if="canWrite()" link type="primary" @click="openEdit(row as Location)">编辑</el-button>
          <el-button v-if="canWrite()" link type="danger" @click="remove(row as Location)">删除</el-button>
        </template>
      </DataTable>
    </el-card>

    <el-dialog v-model="dialog" :title="editingId !== null ? '编辑库位' : '新建库位'" width="520px" destroy-on-close>
      <el-form label-width="100px">
        <el-form-item label="所属仓库">
          <EntitySelect v-model="form.warehouse_id" :api="searchWarehouses" placeholder="搜索并选择仓库" />
        </el-form-item>
        <el-form-item label="编码" required><el-input v-model="form.code" /></el-form-item>
        <el-form-item label="名称" required><el-input v-model="form.name" /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialog = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="save">保存</el-button>
      </template>
    </el-dialog>
  </section>
</template>