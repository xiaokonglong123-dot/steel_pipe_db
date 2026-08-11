<script setup lang="ts">
import { reactive, ref, onMounted, watch } from "vue"
import { ElMessage } from "element-plus"
import { get } from "@/api/client"
import type { ApiPage, OperationLog } from "@/types"

const rows = ref<readonly OperationLog[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const loading = ref(false)
const filters = reactive({ action: "", entity: "", user_id: "" })

async function load(): Promise<void> {
  loading.value = true
  try {
    const params = new URLSearchParams()
    params.set("page", String(page.value))
    params.set("page_size", String(pageSize.value))
    if (filters.action) params.set("action", filters.action)
    if (filters.entity) params.set("entity", filters.entity)
    if (filters.user_id) params.set("user_id", filters.user_id)
    const env = await get<ApiPage<OperationLog>>(`/operation-logs?${params.toString()}`)
    rows.value = env.items
    total.value = env.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally { loading.value = false }
}

onMounted(() => void load())
watch([page, pageSize], () => void load())
</script>

<template>
  <section class="page">
    <div class="heading"><h2>操作日志</h2><el-button @click="load">刷新</el-button></div>
    <el-card>
      <el-form inline>
        <el-form-item label="动作">
          <el-input v-model="filters.action" clearable placeholder="create/update/delete/login..." style="width: 160px" />
        </el-form-item>
        <el-form-item label="对象">
          <el-input v-model="filters.entity" clearable placeholder="item/PO/SO..." style="width: 160px" />
        </el-form-item>
        <el-form-item label="用户 ID">
          <el-input v-model="filters.user_id" clearable style="width: 100px" />
        </el-form-item>
        <el-form-item><el-button type="primary" @click="load">查询</el-button></el-form-item>
      </el-form>
      <el-table :data="rows" v-loading="loading" border>
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="created_at" label="时间" width="180" />
        <el-table-column prop="user_id" label="用户" width="80" />
        <el-table-column prop="action" label="动作" width="120" />
        <el-table-column prop="entity" label="对象" width="120" />
        <el-table-column prop="entity_id" label="对象 ID" width="100" />
        <el-table-column prop="detail" label="详情" show-overflow-tooltip />
        <el-table-column prop="ip_address" label="IP" width="140" />
      </el-table>
      <el-pagination
        :current-page="page"
        :page-size="pageSize"
        :total="total"
        layout="total, prev, pager, next, jumper"
        @current-change="page = $event"
        @size-change="pageSize = $event"
      />
    </el-card>
  </section>
</template>

<style scoped>
.heading { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.heading h2 { margin: 0; }
</style>
