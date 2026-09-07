<script setup lang="ts">
import { onMounted, ref } from "vue"
import { useRouter } from "vue-router"
import { ElMessage } from "element-plus"
import { DataTable, EntitySelect, PageHeader } from "@/components"
import { listChecks, createCheck } from "@/api/inventory"
import { searchLocations } from "@/api/selects"
import { useHasPermission } from "@/composables/useHasPermission"
import type { CheckSession } from "@/types/inventory"

const router = useRouter()
const can = useHasPermission()

const rows = ref<CheckSession[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const loading = ref(false)

const dialog = ref(false)
const locationId = ref<number | null>(null)
const creating = ref(false)

function statusLabel(s: CheckSession["status"]): string {
  if (s === "posted") return "已过账"
  if (s === "counted") return "已盘点"
  return "草稿"
}
function statusTag(s: CheckSession["status"]): "success" | "warning" | "info" {
  if (s === "posted") return "success"
  if (s === "counted") return "warning"
  return "info"
}

async function load(): Promise<void> {
  loading.value = true
  try {
    const r = await listChecks({ page: page.value, page_size: pageSize.value })
    rows.value = [...r.items]
    total.value = r.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

async function create(): Promise<void> {
  if (locationId.value == null) {
    ElMessage.warning("请选择库位")
    return
  }
  creating.value = true
  try {
    await createCheck({ location_id: locationId.value, scope: "all" })
    dialog.value = false
    ElMessage.success("盘点单已创建")
    void load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "创建失败")
  } finally {
    creating.value = false
  }
}

function openCreate(): void {
  locationId.value = null
  dialog.value = true
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <PageHeader title="盘点单" subtitle="选择库位生成盘点单 → 录入实盘数 → 过账调整">
      <el-button v-if="can('stock.write')" type="primary" @click="openCreate">新建盘点</el-button>
    </PageHeader>

    <el-card>
      <DataTable
        :columns="[
          { prop: 'session_no', label: '盘点单号' },
          { prop: 'location_name', label: '库位' },
          { prop: 'status', label: '状态' },
          { prop: 'created_at', label: '创建时间' },
        ]"
        :data="rows"
        :total="total"
        :page="page"
        :page-size="pageSize"
        :loading="loading"
        @page-change="page = $event; load()"
        @page-size-change="pageSize = $event; page = 1; load()"
      >
        <template #cell-status="{ row }">
          <el-tag :type="statusTag((row as CheckSession).status)">{{ statusLabel((row as CheckSession).status) }}</el-tag>
        </template>
        <template #actions="{ row }">
          <el-button link type="primary" @click="router.push(`/inventory/checks/${(row as CheckSession).id}`)">明细</el-button>
        </template>
      </DataTable>
    </el-card>

    <el-dialog v-model="dialog" title="新建盘点" width="440px" destroy-on-close>
      <el-form label-width="80px">
        <el-form-item label="库位" required>
          <EntitySelect v-model="locationId" :api="searchLocations" placeholder="搜索并选择库位" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialog = false">取消</el-button>
        <el-button type="primary" :loading="creating" @click="create">创建</el-button>
      </template>
    </el-dialog>
  </section>
</template>