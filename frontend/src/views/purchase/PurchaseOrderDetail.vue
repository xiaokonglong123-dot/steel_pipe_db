<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue"
import { useRoute, useRouter } from "vue-router"
import { ElMessage, ElMessageBox } from "element-plus"
import { ActionBar, EntitySelect, MoneyText, OrderStatusTag, PageHeader } from "@/components"
import { searchLocations } from "@/api/selects"
import { getPurchaseOrder, actPurchaseOrder, deletePurchaseOrder, receivePurchaseOrder, type ReceivedItem } from "@/api/purchase"
import { useHasPermission } from "@/composables/useHasPermission"
import { ACTION_PERMISSION } from "@/utils/actions"
import { gt, sub } from "@/utils/money"
import type { OrderAction, PurchaseOrderDetail, PurchaseOrderItem } from "@/types/order"

const route = useRoute()
const router = useRouter()
const can = useHasPermission()

const detail = ref<PurchaseOrderDetail | null>(null)
const loading = ref(false)

const id = Number(route.params.id)
const order = computed(() => detail.value?.order ?? null)
const visibleActions = computed(() =>
  (detail.value?.allowed_actions ?? []).filter((a) => can.value(ACTION_PERMISSION[a] ?? "")),
)

async function load(): Promise<void> {
  loading.value = true
  try {
    detail.value = await getPurchaseOrder(id)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

onMounted(() => void load())

async function onAction(action: OrderAction): Promise<void> {
  if (action === "edit") {
    void router.push(`/purchase-orders/${id}/edit`)
    return
  }
  if (action === "delete") {
    await ElMessageBox.confirm("确定删除该采购订单吗？", "提示", { type: "warning" })
    await deletePurchaseOrder(id)
    ElMessage.success("已删除")
    void router.push("/purchase-orders")
    return
  }
  if (action === "receive") {
    openReceive()
    return
  }
  await actPurchaseOrder(id, action as Exclude<OrderAction, "edit" | "delete" | "receive">)
  ElMessage.success("操作成功")
  await load()
}

// —— 收货 ——
interface ReceiveLine {
  item_id: number
  item_name: string
  remaining: string
  quantity: string
  location_id: number | null
}

const receiveDialog = ref(false)
const receiving = ref(false)
const receiveLines = reactive<ReceiveLine[]>([])

function remainingOf(it: PurchaseOrderItem): string {
  return sub(it.quantity, it.received_qty)
}

function openReceive(): void {
  if (!detail.value) return
  receiveLines.splice(
    0,
    receiveLines.length,
    ...detail.value.items
      .filter((it) => gt(remainingOf(it), "0"))
      .map((it) => ({
        item_id: it.item_id,
        item_name: it.item_name,
        remaining: remainingOf(it),
        quantity: remainingOf(it),
        location_id: null,
      })),
  )
  receiveDialog.value = true
}

async function submitReceive(): Promise<void> {
  const items: ReceivedItem[] = []
  for (const line of receiveLines) {
    if (!gt(line.quantity, "0")) continue
    if (line.location_id == null) {
      ElMessage.warning(`请为「${line.item_name}」选择入库库位`)
      return
    }
    if (gt(line.quantity, line.remaining)) {
      ElMessage.warning(`「${line.item_name}」收货数量超过剩余数量 ${line.remaining}`)
      return
    }
    items.push({ item_id: line.item_id, location_id: line.location_id, quantity: line.quantity })
  }
  if (items.length === 0) {
    ElMessage.warning("请输入至少一行的收货数量")
    return
  }
  receiving.value = true
  try {
    await receivePurchaseOrder(id, items)
    ElMessage.success("收货完成，库存已入账")
    receiveDialog.value = false
    await load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "收货失败")
  } finally {
    receiving.value = false
  }
}
</script>

<template>
  <section class="page" v-loading="loading">
    <template v-if="order">
      <PageHeader :title="`采购订单 ${order.order_no}`">
        <ActionBar :actions="visibleActions" @action="onAction" />
      </PageHeader>

      <el-card>
        <el-descriptions :column="3" border>
          <el-descriptions-item label="供应商">{{ order.supplier_name }}</el-descriptions-item>
          <el-descriptions-item label="合同号">{{ order.contract_no ?? "—" }}</el-descriptions-item>
          <el-descriptions-item label="状态">
            <OrderStatusTag :status="order.status" variant="purchase" />
          </el-descriptions-item>
          <el-descriptions-item label="订单日期">{{ order.order_date }}</el-descriptions-item>
          <el-descriptions-item label="总金额">
            <MoneyText :value="order.total_amount" strong /> {{ order.currency }}
          </el-descriptions-item>
          <el-descriptions-item label="备注">{{ order.notes ?? "—" }}</el-descriptions-item>
        </el-descriptions>
      </el-card>

      <el-card class="items">
        <template #header>明细</template>
        <el-table :data="detail!.items" border stripe size="small">
          <el-table-column prop="item_name" label="商品" min-width="200" />
          <el-table-column prop="quantity" label="数量" width="100" />
          <el-table-column label="单价" width="120" align="right">
            <template #default="{ row }"><MoneyText :value="(row as PurchaseOrderItem).unit_price" /></template>
          </el-table-column>
          <el-table-column label="行小计" width="140" align="right">
            <template #default="{ row }"><MoneyText :value="(row as PurchaseOrderItem).total_price" strong /></template>
          </el-table-column>
          <el-table-column prop="received_qty" label="已收货" width="100" />
        </el-table>
      </el-card>
    </template>

    <el-dialog v-model="receiveDialog" title="收货入库" width="720px" destroy-on-close>
      <el-table :data="receiveLines" border size="small">
        <el-table-column prop="item_name" label="商品" min-width="180" />
        <el-table-column prop="remaining" label="剩余数量" width="100" align="right" />
        <el-table-column label="本次收货" width="140">
          <template #default="{ row }">
            <el-input-number v-model="row.quantity" :min="0" :precision="4" :controls="false" style="width: 100%" />
          </template>
        </el-table-column>
        <el-table-column label="入库库位" min-width="220">
          <template #default="{ row }">
            <EntitySelect v-model="row.location_id" :api="searchLocations" placeholder="选择库位" />
          </template>
        </el-table-column>
      </el-table>
      <template #footer>
        <el-button @click="receiveDialog = false">取消</el-button>
        <el-button type="primary" :loading="receiving" @click="submitReceive">确认收货</el-button>
      </template>
    </el-dialog>
  </section>
</template>

<style scoped>
.items { margin-top: 16px; }
</style>