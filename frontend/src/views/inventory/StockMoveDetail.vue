<script setup lang="ts">
import { computed, onMounted, ref } from "vue"
import { useRoute } from "vue-router"
import { ElMessage, ElMessageBox } from "element-plus"
import { PageHeader } from "@/components"
import { getInbound, getOutbound, postInbound, postOutbound } from "@/api/inventory"
import { useHasPermission } from "@/composables/useHasPermission"
import type { InboundDetail, InboundItem, OutboundDetail, OutboundItem } from "@/types/inventory"

const props = defineProps<{ mode: "inbound" | "outbound" }>()
const route = useRoute()
const can = useHasPermission()

const isInbound = props.mode === "inbound"
const id = Number(route.params.id)

const detail = ref<InboundDetail | OutboundDetail | null>(null)
const loading = ref(false)
const posting = ref(false)

const order = computed(() => detail.value?.order ?? null)
const items = computed(() => (detail.value?.items ?? []) as (InboundItem | OutboundItem)[])

const TYPE_LABELS: Record<string, string> = {
  purchase: "采购入库", production: "生产入库", return: "退货入库",
  sales: "销售出库", requisition: "领用出库", other: "其他",
}

const statusLabel = computed(() => {
  const s = order.value?.status
  if (s === "posted") return "已过账"
  if (s === "voided") return "已作废"
  return "草稿"
})

async function load(): Promise<void> {
  loading.value = true
  try {
    detail.value = isInbound ? await getInbound(id) : await getOutbound(id)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

async function post(): Promise<void> {
  await ElMessageBox.confirm("确定过账吗？过账后库存立即变动。", "提示", { type: "warning" })
  posting.value = true
  try {
    if (isInbound) await postInbound(id)
    else await postOutbound(id)
    ElMessage.success("过账成功")
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
    <template v-if="order">
      <PageHeader :title="`${isInbound ? '入库单' : '出库单'} ${order.record_no}`">
        <el-button v-if="can('stock.write') && order.status === 'draft'" type="primary" :loading="posting" @click="post">
          过账
        </el-button>
      </PageHeader>

      <el-card>
        <el-descriptions :column="3" border>
          <el-descriptions-item :label="isInbound ? '入库类型' : '出库类型'">
            {{ TYPE_LABELS[isInbound ? (order as InboundDetail['order']).inbound_type : (order as OutboundDetail['order']).outbound_type] ?? '其他' }}
          </el-descriptions-item>
          <el-descriptions-item label="状态">{{ statusLabel }}</el-descriptions-item>
          <el-descriptions-item label="创建时间">{{ order.created_at }}</el-descriptions-item>
          <el-descriptions-item label="备注">{{ order.notes ?? "—" }}</el-descriptions-item>
        </el-descriptions>
      </el-card>

      <el-card class="items">
        <template #header>明细</template>
        <el-table :data="items" border stripe size="small">
          <el-table-column prop="item_name" label="商品" min-width="200" />
          <el-table-column prop="location_name" label="库位" min-width="180" />
          <el-table-column prop="quantity" label="数量" width="120" align="right" />
        </el-table>
      </el-card>
    </template>
  </section>
</template>

<style scoped>
.items { margin-top: 16px; }
</style>