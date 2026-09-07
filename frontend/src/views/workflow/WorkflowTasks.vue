<script setup lang="ts">
import { onMounted, ref } from "vue"
import { ElMessage } from "element-plus"
import { PageHeader } from "@/components"
import { listMyTasks, completeTask } from "@/api/workflow"
import { TASK_STATUS } from "@/utils/actions"
import type { WorkflowTask } from "@/types/workflow"

const tasks = ref<WorkflowTask[]>([])
const loading = ref(false)

const dialog = ref(false)
const current = ref<WorkflowTask | null>(null)
const action = ref("approve")
const comment = ref("")
const completing = ref(false)

const taskMeta = (s: string) => TASK_STATUS[s] ?? { label: s, type: "info" as const }

async function load(): Promise<void> {
  loading.value = true
  try {
    tasks.value = [...(await listMyTasks())]
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

function openComplete(task: WorkflowTask): void {
  current.value = task
  action.value = "approve"
  comment.value = ""
  dialog.value = true
}

async function submit(): Promise<void> {
  if (!current.value) return
  completing.value = true
  try {
    await completeTask(current.value.id, {
      action: action.value,
      ...(comment.value ? { comment: comment.value } : {}),
    })
    ElMessage.success("任务已完成")
    dialog.value = false
    void load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "处理失败")
  } finally {
    completing.value = false
  }
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <PageHeader title="待办任务" subtitle="我的待办审批任务" />
    <el-card>
      <el-table :data="tasks" v-loading="loading" border stripe>
        <el-table-column prop="id" label="任务 ID" width="90" />
        <el-table-column prop="state_key" label="状态节点" width="160" />
        <el-table-column label="状态" width="120">
          <template #default="{ row }">
            <el-tag :type="taskMeta((row as WorkflowTask).status).type">{{ taskMeta((row as WorkflowTask).status).label }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" min-width="180" />
        <el-table-column label="操作" width="120">
          <template #default="{ row }">
            <el-button
              v-if="(row as WorkflowTask).status === 'pending'"
              link
              type="primary"
              @click="openComplete(row as WorkflowTask)"
            >
              处理
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-dialog v-model="dialog" title="处理任务" width="440px" destroy-on-close>
      <el-form label-width="80px">
        <el-form-item label="动作">
          <el-select v-model="action" style="width: 100%">
            <el-option value="approve" label="审批通过" />
            <el-option value="reject" label="驳回" />
          </el-select>
        </el-form-item>
        <el-form-item label="意见">
          <el-input v-model="comment" type="textarea" :rows="2" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialog = false">取消</el-button>
        <el-button type="primary" :loading="completing" @click="submit">提交</el-button>
      </template>
    </el-dialog>
  </section>
</template>