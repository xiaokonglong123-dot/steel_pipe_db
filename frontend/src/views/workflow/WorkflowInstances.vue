<script setup lang="ts">
import { computed, onMounted, ref } from "vue"
import { ElMessage } from "element-plus"
import { listInstances } from "@/api/workflow"
import { INSTANCE_STATUS } from "@/utils/actions"
import type { WorkflowInstance } from "@/types/workflow"

const rows = ref<WorkflowInstance[]>([])
const loading = ref(false)
const filterStatus = ref<"">("")

const statusMeta = computed(() => (s: string) => INSTANCE_STATUS[s] ?? { label: s, type: "info" as const })

async function load(): Promise<void> {
  loading.value = true
  try {
    rows.value = [...(await listInstances(filterStatus.value ? { status: filterStatus.value } : {}))]
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

function stateLabel(s: string): string {
  const map: Record<string, string> = {
    draft: "草稿", submitted: "待审批", senior_review: "高级复核",
    approved: "已审批", rejected: "已驳回", received: "已收货", shipped: "已发货",
  }
  return map[s] ?? s
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <div class="heading">
      <h2>流程实例</h2>
      <el-select v-model="filterStatus" clearable placeholder="全部状态" style="width: 160px" @change="load">
        <el-option value="active" label="进行中" />
        <el-option value="completed" label="已完成" />
        <el-option value="cancelled" label="已取消" />
      </el-select>
    </div>

    <el-card>
      <el-table :data="rows" v-loading="loading" border stripe>
        <el-table-column prop="id" label="实例 ID" width="90" />
        <el-table-column prop="business_type" label="业务类型" width="140" />
        <el-table-column prop="business_id" label="业务 ID" width="100" />
        <el-table-column label="当前状态" width="140">
          <template #default="{ row }">{{ stateLabel((row as WorkflowInstance).current_state) }}</template>
        </el-table-column>
        <el-table-column label="状态" width="120">
          <template #default="{ row }">
            <el-tag :type="statusMeta((row as WorkflowInstance).status).type">
              {{ statusMeta((row as WorkflowInstance).status).label }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="updated_at" label="更新时间" />
      </el-table>
    </el-card>
  </section>
</template>

<style scoped>
.heading { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.heading h2 { margin: 0; }
</style>