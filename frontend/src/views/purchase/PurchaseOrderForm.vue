<script setup lang="ts">
import { onMounted, ref } from "vue"
import { useRoute, useRouter } from "vue-router"
import { ElMessage } from "element-plus"
import { EntitySelect, OrderLineEditor, PageHeader } from "@/components"
import { searchSuppliers, searchItems } from "@/api/selects"
import { createPurchaseOrder, updatePurchaseOrder, getPurchaseOrder } from "@/api/purchase"
import type { LineField } from "@/components/domain/OrderLineEditor.vue"
import type { PurchaseOrderPayload } from "@/types/order"

const route = useRoute()
const router = useRouter()

const rawId = route.params.id
const id = typeof rawId === "string" && rawId !== "" ? Number(rawId) : null
const isEdit = id !== null && Number.isFinite(id)

const supplierId = ref<number | null>(null)
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
    const d = await getPurchaseOrder(id)
    supplierId.value = d.order.supplier_id
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

async function save(): Promise<void> {
  if (supplierId.value == null) {
    ElMessage.warning("请选择供应商")
    return
  }
  if (!editor.value?.validate()) return
  saving.value = true
  try {
    const payload: PurchaseOrderPayload = {
      supplier_id: supplierId.value,
      order_date: orderDate.value,
      ...(contractNo.value ? { contract_no: contractNo.value } : {}),
      ...(notes.value ? { notes: notes.value } : {}),
      items: lines.value
        .filter((l) => l.item_id != null)
        .map((l) => ({
          item_id: l.item_id as number,
          quantity: l.quantity,
          ...(l.unit_price !== undefined ? { unit_price: l.unit_price } : {}),
          ...(l.notes ? { notes: l.notes } : {}),
        })),
    }
    const saved = isEdit && id !== null
      ? await updatePurchaseOrder(id, payload)
      : await createPurchaseOrder(payload)
    ElMessage.success(isEdit ? "订单已更新" : "订单已创建")
    void router.push(`/purchase-orders/${saved.id}`)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "保存失败")
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <section class="page" v-loading="loading">
    <PageHeader :title="isEdit ? '编辑采购订单' : '新建采购订单'" />
    <el-card>
      <el-form label-width="100px">
        <el-form-item label="供应商" required>
          <EntitySelect v-model="supplierId" :api="searchSuppliers" placeholder="搜索并选择供应商" />
        </el-form-item>
        <el-form-item label="订单日期" required>
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
      <template #header>订单明细</template>
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