<script setup lang="ts">
import { onMounted, ref } from "vue"
import { ElMessage } from "element-plus"
import { DataTable, PageHeader } from "@/components"
import { listOperationLogs } from "@/api/auth"
import type { OperationLog } from "@/types/auth"

const rows = ref<OperationLog[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const loading = ref(false)

async function load(): Promise<void> {
  loading.value = true
  try {
    const r = await listOperationLogs(page.value, pageSize.value)
    rows.value = [...r.items]
    total.value = r.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <PageHeader title="操作日志" subtitle="审计用户动作（管理员）" />
    <el-card>
      <DataTable
        :columns="[
          { prop: 'created_at', label: '时间' },
          { prop: 'user_id', label: '用户' },
          { prop: 'action', label: '动作' },
          { prop: 'target_type', label: '对象类型' },
          { prop: 'target_id', label: '对象 ID' },
          { prop: 'ip_address', label: 'IP' },
        ]"
        :data="rows"
        :total="total"
        :page="page"
        :page-size="pageSize"
        :loading="loading"
        @page-change="page = $event; load()"
        @page-size-change="pageSize = $event; page = 1; load()"
      />
    </el-card>
  </section>
</template>