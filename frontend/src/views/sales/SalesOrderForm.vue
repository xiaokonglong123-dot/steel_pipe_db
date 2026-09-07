<script setup lang="ts">
import { onMounted, ref } from "vue"
import { useRoute, useRouter } from "vue-router"
import { ElMessage } from "element-plus"
import { EntitySelect, OrderLineEditor, PageHeader } from "@/components"
import { searchCustomers, searchItems } from "@/api/selects"
import { createSalesOrder, updateSalesOrder, getSalesOrder } from "@/api/sales"
import { getAvailable } from "@/api/inventory"
import { add, gt } from "@/utils/money"
import type { LineField } from "@/components/domain/OrderLineEditor.vue"
import type { SalesOrderPayload } from "@/types/order"

const route = useRoute()
const router = useRouter()

const rawId = route.params.id
const id = typeof rawId === "string" && rawId !== "" ? Number(rawId) : null
const isEdit = id !== null && Number.isFinite(id)

const customerId = ref<number | null>(null)
const orderDate = ref(new Date().toISOString().slice(0, 10))
const contractNo = ref("")
const notes = ref("")
const lines = ref<LineField[]>([{ item_id: null, quantity: "1", unit_price: "0" }])
const saving = ref(false)
const loading = ref(false)
const editor = ref<InstanceType<typeof OrderLineEditor> | null>(null)

onMounted(async () => {
  if (!isEdit || id === null) return
  loading.value = true
  try {
    const d = await getSalesOrder(id)
    customerId.value = d.order.customer_id
    orderDate.value = d.order.order_date.slice(0, 10)
    contractNo.value = d.order.contract_no ?? ""
    notes.value = d.order.notes ?? ""
    lines.value = d.items.map((it) => ({
      item_id: it.item_id,
      quantity: it.quantity,
      unit_price: it.unit_price ?? "0",
      ...(it.notes ? { notes: it.notes } : {}),
    }))
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
})

/** 提交前 ATP 预检：对每项商品汇总需求量并与可用量比较（后端 submit 时仍会权威校验）。 */
async function checkAtp(): Promise<boolean> {
  const demand = new Map<number, string>()
  for (const l of lines.value) {
    if (l.item_id == null) continue
    demand.set(l.item_id, demand.has(l.item_id) ? add(demand.get(l.item_id) as string, l.quantity) : l.quantity)
  }
  for (const [itemId, qty] of demand) {
    const a = await getAvailable(itemId)
    if (gt(qty, a.available_qty)) {
      ElMessage.warning(`库存不足：商品 #${itemId} 可用 ${a.available_qty}，需求 ${qty}`)
      return false
    }
  }
  return true
}

async function save(): Promise<void> {
  if (customerId.value == null) {
    ElMessage.warning("请选择客户")
    return
  }
  if (!editor.value?.validate()) return
  if (!(await checkAtp())) return
  saving.value = true
  try {
    const payload: SalesOrderPayload = {
      customer_id: customerId.value,
      order_date: orderDate.value,
      ...(contractNo.value ? { contract_no: contractNo.value } : {}),
      ...(notes.value ? { notes: notes.value } : {}),
      items: lines.value
        .filter((l) => l.item_id != null)
        .map((l) => ({
          item_id: l.item_id as number,
          quantity: l.quantity,
          unit_price: l.unit_price ?? "0",
          ...(l.notes ? { notes: l.notes } : {}),
        })),
    }
    const saved = isEdit && id !== null
      ? await updateSalesOrder(id, payload)
      : await createSalesOrder(payload)
    ElMessage.success(isEdit ? "订单已更新" : "订单已创建")
    void router.push(`/sales-orders/${saved.id}`)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "保存失败")
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <section class="page" v-loading="loading">
    <PageHeader :title="isEdit ? '编辑销售订单' : '新建销售订单'" />
    <el-card>
      <el-form label-width="100px">
        <el-form-item label="客户" required>
          <EntitySelect v-model="customerId" :api="searchCustomers" placeholder="搜索并选择客户" />
        </el-form-item>
        <el-form-item label="订单日期">
          <el-date-picker v-model="orderDate" type="date" value-format="YYYY-MM-DD" style="width: 200px" />
        </el-form-item>
        <el-form-item label="合同号">
          <el-input v-model="contractNo" placeholder="如 HT-2026-001" style="width: 240px" />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="notes" type="textarea" :rows="2" />
        </el-form-item>
      </el-form>
    </el-card>

    <el-card class="lines">
      <template #header>订单明细（提交前校验可用量）</template>
      <OrderLineEditor ref="editor" v-model:lines="lines" :item-api="searchItems" :show-price="true" currency="CNY" />
    </el-card>

    <div class="footer">
      <el-button @click="router.back()">返回</el-button>
      <el-button type="primary" :loading="saving" @click="save">保存</el-button>
    </div>
  </section>
</template>

<style scoped>
.lines { margin-top: 16px; }
.footer { margin-top: 16px; display: flex; justify-content: flex-end; gap: 8px; }
</style>