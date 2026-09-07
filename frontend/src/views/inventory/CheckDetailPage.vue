<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue"
import { useRoute } from "vue-router"
import { ElMessage } from "element-plus"
import { MoneyText, PageHeader } from "@/components"
import { getCheck, recordActual, postCheck } from "@/api/inventory"
import { useHasPermission } from "@/composables/useHasPermission"
import type { CheckDetail, CheckDetailRow } from "@/types/inventory"

const route = useRoute()
const can = useHasPermission()

const id = Number(route.params.id)
const detail = ref<CheckDetail | null>(null)
const loading = ref(false)
const editing = reactive<Record<number, string>>({})
const busyId = ref<number | null>(null)
const posting = ref(false)

const session = computed(() => detail.value?.session ?? null)
const details = computed(() => detail.value?.details ?? [])

const editable = computed(() => {
  const s = session.value?.status
  return s === "draft" || s === "counted"
})

async function load(): Promise<void> {
  loading.value = true
  try {
    detail.value = await getCheck(id)
    for (const d of detail.value.details) {
      editing[d.id] = d.actual_qty ?? d.system_qty
    }
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

function diffOf(d: CheckDetailRow): string {
  return d.diff_qty ?? "0"
}

async function saveActual(d: CheckDetailRow): Promise<void> {
  const qty = editing[d.id]
  if (qty === undefined || qty === "") {
    ElMessage.warning("请输入实盘数量")
    return
  }
  busyId.value = d.id
  try {
    await recordActual(id, d.id, qty)
    ElMessage.success("已保存实盘数")
    await load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "保存失败")
  } finally {
    busyId.value = null
  }
}

async function post(): Promise<void> {
  posting.value = true
  try {
    await postCheck(id)
    ElMessage.success("过账成功，库存已按差异调整")
    await load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "过账失败")
  } finally {
    posting.value = false
  }
}

onMounted(() => void load())
</script>

<template>
  <section class="page" v-loading="loading">
    <template v-if="session">
      <PageHeader :title="`盘点单 ${session.session_no}`">
        <el-button v-if="can('stock.write') && session.status !== 'posted'" type="primary" :loading="posting" @click="post">
          过账调整
        </el-button>
      </PageHeader>

      <el-card>
        <el-descriptions :column="2" border>
          <el-descriptions-item label="库位">{{ session.location_name ?? session.location_id }}</el-descriptions-item>
          <el-descriptions-item label="状态">{{ session.status }}</el-descriptions-item>
          <el-descriptions-item label="创建时间">{{ session.created_at }}</el-descriptions-item>
        </el-descriptions>
      </el-card>

      <el-card class="items">
        <template #header>实盘明细（系统数 vs 实盘数）</template>
        <el-table :data="details" border stripe size="small">
          <el-table-column prop="item_name" label="商品" min-width="180" />
          <el-table-column prop="sku" label="SKU" width="140" />
          <el-table-column label="系统数量" width="120" align="right">
            <template #default="{ row }"><MoneyText :value="(row as CheckDetailRow).system_qty" kind="qty" /></template>
          </el-table-column>
          <el-table-column label="实盘数量" width="180">
            <template #default="{ row }">
              <el-input-number
                :model-value="Number(editing[(row as CheckDetailRow).id] ?? 0)"
                :min="0"
                :precision="4"
                :controls="false"
                :disabled="!editable"
                style="width: 120px"
                @update:model-value="(v?: number) => { editing[(row as CheckDetailRow).id] = String(v ?? 0) }"
              />
            </template>
          </el-table-column>
          <el-table-column label="差异" width="120" align="right">
            <template #default="{ row }">
              <span :style="{ color: Number(diffOf(row as CheckDetailRow)) !== 0 ? 'var(--el-color-warning)' : '' }">
                <MoneyText :value="diffOf(row as CheckDetailRow)" kind="qty" />
              </span>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="90" align="center">
            <template #default="{ row }">
              <el-button
                v-if="editable"
                link
                type="primary"
                :loading="busyId === (row as CheckDetailRow).id"
                @click="saveActual(row as CheckDetailRow)"
              >
                保存
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-card>
    </template>
  </section>
</template>

<style scoped>
.items { margin-top: 16px; }
</style>