<script setup lang="ts">
import { ref, reactive, onMounted, computed } from "vue"
import { useRoute, useRouter } from "vue-router"
import { ElMessage } from "element-plus"
import { get, post } from "@/api/client"
import type { CheckSession, CheckDetail } from "@/types"

const route = useRoute()
const router = useRouter()
const sessionId = computed(() => Number(route.params.id))
const session = ref<CheckSession | null>(null)
const details = ref<readonly CheckDetail[]>([])
const editing = reactive<Record<number, number>>({})
const loading = ref(false)
const busy = ref(false)

async function load(): Promise<void> {
  loading.value = true
  try {
    const result = await get<CheckSession>(`/check-records/${sessionId.value}`)
    session.value = result
    details.value = result.details ?? []
    for (const d of result.details ?? []) {
      editing[d.id] = d.actual_qty ?? d.system_qty
    }
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally { loading.value = false }
}

function canEdit(): boolean {
  return session.value?.status === "draft" || session.value?.status === "counted"
}

async function recordActual(d: CheckDetail): Promise<void> {
  const qty = editing[d.id]
  if (qty === undefined || qty === null) return
  busy.value = true
  try {
    await post(`/inventory/checks/${sessionId.value}/count`, { detail_id: d.id, actual_qty: qty })
    ElMessage.success("已保存实盘数")
    await load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "保存失败")
  } finally { busy.value = false }
}

async function postSession(): Promise<void> {
  if (!session.value || session.value.status !== "counted") return
  busy.value = true
  try {
    await post(`/check-records/${sessionId.value}/post`)
    ElMessage.success("过账成功")
    await load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "过账失败")
  } finally { busy.value = false }
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <div class="heading">
      <div>
        <h2>盘点点明细 #{{ sessionId }}</h2>
        <p v-if="session">状态：{{ session.status }} · 库位：{{ session.location_id }} · 范围：{{ session.scope }}</p>
      </div>
      <div>
        <el-button @click="router.push('/inventory/checks')">返回列表</el-button>
        <el-button v-if="session && session.status === 'counted'" type="primary" :disabled="busy" @click="postSession">过账</el-button>
      </div>
    </div>
    <el-card>
      <el-table :data="details" v-loading="loading" border>
        <el-table-column prop="id" label="明细 ID" width="100" />
        <el-table-column prop="item_id" label="商品 ID" width="120" />
        <el-table-column prop="item_sku" label="SKU" width="160" />
        <el-table-column prop="item_name" label="商品名称" />
        <el-table-column prop="system_qty" label="系统数量" align="right" width="120" />
        <el-table-column label="实盘数量" align="right" width="180">
          <template #default="{ row }">
            <el-input-number
              v-if="canEdit()"
              v-model="editing[(row as CheckDetail).id]"
              :min="0"
              :precision="2"
              controls-position="right"
            />
            <span v-else>{{ (row as CheckDetail).actual_qty ?? "—" }}</span>
          </template>
        </el-table-column>
        <el-table-column label="差异" align="right" width="120">
          <template #default="{ row }">
            <span :class="{ bad: (row as CheckDetail).diff_qty && (row as CheckDetail).diff_qty !== 0 }">
              {{ (row as CheckDetail).diff_qty ?? "—" }}
            </span>
          </template>
        </el-table-column>
        <el-table-column v-if="canEdit()" label="操作" width="100">
          <template #default="{ row }">
            <el-button link type="primary" @click="recordActual(row as CheckDetail)">录入</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </section>
</template>

<style scoped>
.heading { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 16px; }
.heading h2 { margin: 0 0 8px; }
.heading p { margin: 0; color: var(--muted); }
.bad { color: var(--el-color-danger); font-weight: bold; }
</style>
