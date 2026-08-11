<script setup lang="ts">
import { ref } from "vue"
import { ElMessage } from "element-plus"
import CrudList from "@/components/CrudList.vue"
import type { ImportReport, ApiEnvelope } from "@/types"

const columns = [
  { prop: "sku", label: "SKU", required: true },
  { prop: "name", label: "名称", required: true },
  { prop: "category", label: "分类" },
  { prop: "unit", label: "单位" },
  { prop: "status", label: "状态" },
] as const
const searchFields = columns.filter((item) => ["sku", "name", "status"].includes(item.prop))

const importDialog = ref(false)
const importReport = ref<ImportReport | null>(null)
const importFile = ref<File | null>(null)
const importing = ref(false)
const refreshKey = ref(0)

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
    const auth = JSON.parse(localStorage.getItem("auth") ?? "{}")
    const token = auth?.auth_token
    const form = new FormData()
    form.append("file", importFile.value)
    const res = await fetch("/api/v1/items/import", {
      method: "POST",
      headers: token ? { Authorization: `Bearer ${token}` } : {},
      body: form,
    })
    if (!res.ok) throw new Error("导入失败")
    const env = (await res.json()) as ApiEnvelope<ImportReport>
    importReport.value = env.data
    importDialog.value = false
    ElMessage.success("导入完成")
    refreshKey.value++
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "导入失败")
  } finally {
    importing.value = false
  }
}

function closeReport(): void {
  importReport.value = null
}
</script>

<template>
  <div :key="refreshKey">
    <div class="heading">
      <!-- CrudList 内部含标题，这里只放右上角按钮 -->
      <div></div>
      <el-button type="primary" @click="importDialog = true">CSV 导入</el-button>
    </div>
    <CrudList
      title="商品"
      endpoint="/items"
      :columns="columns"
      :search-fields="searchFields"
      permission="item.read"
      write-permission="item.write"
    />

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

    <el-dialog v-model="importReport" :show-close="true" title="导入报告" width="480px" @close="closeReport">
      <el-descriptions v-if="importReport" :column="1" border>
        <el-descriptions-item label="总行数">{{ importReport.total }}</el-descriptions-item>
        <el-descriptions-item label="成功">{{ importReport.succeeded }}</el-descriptions-item>
        <el-descriptions-item label="失败">{{ importReport.failed }}</el-descriptions-item>
      </el-descriptions>
      <p v-if="importReport && importReport.failed > 0">失败行（如 SKU 重复、必填空）已自动跳过。</p>
    </el-dialog>
  </div>
</template>

<style scoped>
.heading { display: flex; justify-content: flex-end; margin-bottom: 8px; }
.upload-tip { color: var(--el-text-color-secondary); font-size: 12px; margin-top: 4px; }
</style>
