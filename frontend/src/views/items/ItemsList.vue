<script setup lang="ts">
import MasterDataCrud from "@/components/common/MasterDataCrud.vue"
import type { MasterColumn } from "@/components/common/MasterDataCrud.vue"
import { listItems, createItem, updateItem, deleteItem } from "@/api/catalog"
import { useAuthStore } from "@/stores/auth"
import { ElMessage } from "element-plus"
import { ref } from "vue"
import type { ItemPayload, ImportReport } from "@/types/catalog"
import type { Page } from "@/types/common"
import type { ApiEnvelope } from "@/types/common"

type Row = Record<string, unknown>

const columns: readonly MasterColumn[] = [
  { prop: "sku", label: "SKU", required: true },
  { prop: "name", label: "名称", required: true },
  { prop: "category", label: "分类" },
  { prop: "unit", label: "单位" },
  { prop: "spec", label: "规格" },
  { prop: "status", label: "状态", formType: "select", options: [
    { value: "active", label: "可交易 (active)" },
    { value: "draft", label: "草稿 (draft)" },
    { value: "disabled", label: "停用 (disabled)" },
  ] },
]

async function fetchPage(q: { page: number; page_size: number } & Record<string, string>): Promise<Page<Row>> {
  const r = await listItems({
    page: q.page,
    page_size: q.page_size,
    ...(q.sku ? { sku: q.sku } : {}),
    ...(q.name ? { name: q.name } : {}),
    ...(q.status ? { status: q.status } : {}),
  })
  return r as unknown as Page<Row>
}

const createRow = (row: Row) => createItem(row as unknown as ItemPayload)
const updateRow = (id: number, row: Row) => updateItem(id, row as unknown as ItemPayload)
const deleteRow = (id: number) => deleteItem(id)

// —— CSV 导入（保留 v2 能力）——
const importDialog = ref(false)
const importReport = ref<ImportReport | null>(null)
const importFile = ref<File | null>(null)
const importing = ref(false)

function onFileChange(file: File): void {
  importFile.value = file
}

async function doImport(): Promise<void> {
  if (!importFile.value) {
    ElMessage.warning("请先选择 CSV 文件")
    return
  }
  importing.value = true
  try {
    const auth = useAuthStore()
    const form = new FormData()
    form.append("file", importFile.value)
    const res = await fetch("/api/items/import", {
      method: "POST",
      headers: auth.auth_token ? { Authorization: `Bearer ${auth.auth_token}` } : {},
      body: form,
    })
    if (!res.ok) throw new Error("导入失败")
    const envelope = (await res.json()) as ApiEnvelope<ImportReport>
    importReport.value = envelope.data
    importDialog.value = false
    ElMessage.success("导入完成")
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "导入失败")
  } finally {
    importing.value = false
  }
}
</script>

<template>
  <MasterDataCrud
    title="商品"
    subtitle="商品 SKU 主数据（新建为草稿，编辑后可激活）"
    :columns="columns"
    :search-fields="['sku', 'name', 'status']"
    :fetch-page="fetchPage"
    :create-fn="createRow"
    :update-fn="updateRow"
    :delete-fn="deleteRow"
    write-permission="item.write"
  >
    <template #header-actions>
      <el-button @click="importDialog = true">CSV 导入</el-button>
    </template>
  </MasterDataCrud>

  <el-dialog v-model="importDialog" title="CSV 导入商品" width="500px">
    <el-form label-width="120px">
      <el-form-item label="CSV 文件">
        <el-upload
          :auto-upload="false"
          :limit="1"
          :on-change="(file: any) => onFileChange(file.raw)"
          accept=".csv,text/csv"
        >
          <el-button>选择文件</el-button>
          <template #tip>
            <div class="upload-tip">表头：sku,name,category,unit,spec（category/unit/spec 可空）</div>
          </template>
        </el-upload>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="importDialog = false">取消</el-button>
      <el-button type="primary" :loading="importing" @click="doImport">导入</el-button>
    </template>
  </el-dialog>

  <el-dialog v-model="importReport" title="导入报告" width="480px" @close="importReport = null">
    <el-descriptions v-if="importReport" :column="1" border>
      <el-descriptions-item label="总行数">{{ importReport.total }}</el-descriptions-item>
      <el-descriptions-item label="成功">{{ importReport.succeeded }}</el-descriptions-item>
      <el-descriptions-item label="失败">{{ importReport.failed }}</el-descriptions-item>
    </el-descriptions>
    <p v-if="importReport && importReport.failed > 0">失败行（SKU 重复、必填缺失）已自动跳过。</p>
  </el-dialog>
</template>

<style scoped>
.upload-tip { color: var(--el-text-color-secondary); font-size: 12px; margin-top: 4px; }
</style>