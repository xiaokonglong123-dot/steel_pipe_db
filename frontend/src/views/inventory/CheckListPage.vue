<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useRouter } from "vue-router"
import { ElMessage } from "element-plus"
import { get, post } from "@/api/client"
import type { CheckSession, ApiPage } from "@/types"

const router = useRouter()
const rows = ref<readonly CheckSession[]>([])
const total = ref(0); const page = ref(1); const pageSize = ref(20); const loading = ref(false)
const dialog = ref(false)
const form = ref<{ location_id: number; scope: string }>({ location_id: 0, scope: "all" })

async function load(): Promise<void> {
  loading.value = true
  try {
    const result = await get<ApiPage<CheckSession>>(`/check-records?page=${page.value}&page_size=${pageSize.value}`)
    rows.value = result.items; total.value = result.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally { loading.value = false }
}

function statusLabel(s: CheckSession["status"]): string {
  if (s === "posted") return "已过账"
  if (s === "counted") return "已盘点"
  return "草稿"
}

async function create(): Promise<void> {
  if (!form.value.location_id) { ElMessage.warning("需选择库位"); return }
  try {
    await post("/check-records", form.value)
    dialog.value = false
    ElMessage.success("创建成功")
    await load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "创建失败")
  }
}

function goToDetail(id: number): void {
  router.push(`/inventory/checks/${id}`)
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <div class="heading"><h2>盘点单</h2><el-button type="primary" @click="dialog = true">新建盘点</el-button></div>
    <el-card>
      <el-table :data="rows" v-loading="loading" border>
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="location_id" label="库位 ID" width="120" />
        <el-table-column prop="scope" label="范围" width="120" />
        <el-table-column label="状态" width="120">
          <template #default="{ row }">
            <el-tag>{{ statusLabel((row as CheckSession).status) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" />
        <el-table-column label="操作" width="120">
          <template #default="{ row }"><el-button link type="primary" @click="goToDetail((row as CheckSession).id)">明细</el-button></template>
        </el-table-column>
      </el-table>
      <el-pagination :current-page="page" :page-size="pageSize" :total="total" layout="total, prev, pager, next" @current-change="page = $event" />
    </el-card>
    <el-dialog v-model="dialog" title="新建盘点" width="400px">
      <el-form :model="form" label-width="80px">
        <el-form-item label="库位 ID"><el-input-number v-model="form.location_id" :min="0" controls-position="right" /></el-form-item>
        <el-form-item label="范围">
          <el-select v-model="form.scope">
            <el-option value="all" label="全部" />
            <el-option value="sku" label="按 SKU" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer><el-button @click="dialog = false">取消</el-button><el-button type="primary" @click="create">创建</el-button></template>
    </el-dialog>
  </section>
</template>

<style scoped>
.heading { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.heading h2 { margin: 0; }
</style>
