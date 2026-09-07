<script setup lang="ts">
import { computed, onMounted, ref } from "vue"
import { useRouter } from "vue-router"
import { ElMessage, ElMessageBox } from "element-plus"
import { DataTable, PageHeader } from "@/components"
import { listInbounds, listOutbounds, postInbound, postOutbound } from "@/api/inventory"
import { useHasPermission } from "@/composables/useHasPermission"
import type { InboundOrder, OutboundOrder } from "@/types/inventory"

const props = defineProps<{ mode: "inbound" | "outbound" }>()
const router = useRouter()
const can = useHasPermission()

const isInbound = props.mode === "inbound"
type Row = InboundOrder | OutboundOrder

const rows = ref<Row[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const loading = ref(false)

const title = isInbound ? "入库记录" : "出库记录"

const TYPE_LABELS: Record<string, string> = {
  purchase: "采购入库", production: "生产入库", return: "退货入库",
  sales: "销售出库", requisition: "领用出库", other: "其他",
}

const statusLabel = computed(() => (s: string) => {
  if (s === "posted") return "已过账"
  if (s === "voided") return "已作废"
  return "草稿"
})

const statusTag = computed(() => (s: string) => {
  if (s === "posted") return "success" as const
  if (s === "voided") return "info" as const
  return "warning" as const
})

function typeLabel(r: Row): string {
  const t = isInbound ? (r as InboundOrder).inbound_type : (r as OutboundOrder).outbound_type
  return TYPE_LABELS[t] ?? t
}

async function load(): Promise<void> {
  loading.value = true
  try {
    const r = isInbound
      ? await listInbounds({ page: page.value, page_size: pageSize.value })
      : await listOutbounds({ page: page.value, page_size: pageSize.value })
    rows.value = [...r.items]
    total.value = r.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

function goDetail(id: number): void {
  void router.push(`/inventory/${isInbound ? "inbound" : "outbound"}/${id}`)
}

async function postRow(row: Row): Promise<void> {
  await ElMessageBox.confirm(`确定过账「${row.record_no}」吗？过账后库存立即${isInbound ? "增加" : "扣减"}。`, "提示", { type: "warning" })
  try {
    if (isInbound) await postInbound((row as InboundOrder).id)
    else await postOutbound((row as OutboundOrder).id)
    ElMessage.success("过账成功")
    await load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "过账失败")
  }
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <PageHeader :title="title" subtitle="创建后需在详情或列表过账，库存才会变动">
      <el-button v-if="can('stock.write')" type="primary" @click="router.push(`/inventory/${isInbound ? 'inbound' : 'outbound'}/new`)">
        新建{{ isInbound ? "入库" : "出库" }}
      </el-button>
    </PageHeader>

    <el-card>
      <DataTable
        :columns="[
          { prop: 'record_no', label: '单号' },
          { prop: 'created_at', label: '创建时间' },
          { prop: 'status', label: '状态' },
        ]"
        :data="rows"
        :total="total"
        :page="page"
        :page-size="pageSize"
        :loading="loading"
        @page-change="page = $event; load()"
        @page-size-change="pageSize = $event; page = 1; load()"
      >
        <template #cell-record_no="{ row }">
          {{ (row as Row).record_no }}
          <el-tag size="small" type="info" style="margin-left: 6px">{{ typeLabel(row as Row) }}</el-tag>
        </template>
        <template #cell-status="{ row }">
          <el-tag :type="statusTag((row as Row).status)">{{ statusLabel((row as Row).status) }}</el-tag>
        </template>
        <template #actions="{ row }">
          <el-button link type="primary" @click="goDetail((row as Row).id)">查看</el-button>
          <el-button
            v-if="can('stock.write') && (row as Row).status === 'draft'"
            link
            type="success"
            @click="postRow(row as Row)"
          >
            过账
          </el-button>
        </template>
      </DataTable>
    </el-card>
  </section>
</template>