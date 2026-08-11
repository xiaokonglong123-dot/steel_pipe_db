<script setup lang="ts">
import { reactive } from "vue"
import { ElMessage } from "element-plus"
import { post } from "@/api/client"

const props = withDefaults(defineProps<{ readonly endpoint?: string; readonly partyField?: "supplier_id" | "customer_id" }>(), { endpoint: "/purchase-orders", partyField: "supplier_id" })
const form = reactive({ supplier_id: "", customer_id: "", expected_date: "", items: [{ item_id: "", quantity: 1, unit_price: 0 }] })
function add(): void { form.items.push({ item_id: "", quantity: 1, unit_price: 0 }) }
async function save(): Promise<void> {
  const payload = props.partyField === "customer_id" ? { customer_id: form.customer_id, expected_date: form.expected_date, items: form.items } : { supplier_id: form.supplier_id, expected_date: form.expected_date, items: form.items }
  await post(props.endpoint, payload)
  ElMessage.success("订单已创建")
}
</script>
<template><el-card><template #header>{{ props.partyField === "customer_id" ? "销售订单" : "采购订单" }}</template><el-form :model="form" label-width="100px"><el-form-item :label="props.partyField === 'customer_id' ? '客户 ID' : '供应商 ID'"><el-input v-if="props.partyField === 'customer_id'" v-model="form.customer_id" /><el-input v-else v-model="form.supplier_id" /></el-form-item><el-form-item label="预计日期"><el-date-picker v-model="form.expected_date" type="date" value-format="YYYY-MM-DD" /></el-form-item></el-form><el-divider>订单明细</el-divider><el-form v-for="(item, index) in form.items" :key="index" inline><el-form-item label="商品 ID"><el-input v-model="item.item_id" /></el-form-item><el-form-item label="数量"><el-input-number v-model="item.quantity" :min="1" /></el-form-item><el-form-item label="单价"><el-input-number v-model="item.unit_price" :min="0" /></el-form-item></el-form><el-button @click="add">添加明细</el-button><el-button type="primary" @click="save">保存</el-button></el-card></template>
