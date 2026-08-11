<script setup lang="ts">
import CrudList from "@/components/CrudList.vue"
const columns = [{ prop: "order_no", label: "订单号" }, { prop: "supplier_id", label: "供应商 ID" }, { prop: "status", label: "状态" }, { prop: "expected_date", label: "预计日期" }, { prop: "total_amount", label: "总金额" }] as const
</script>
<template><CrudList title="采购订单" endpoint="/purchase-orders" :columns="columns" :search-fields="[columns[2]]" permission="order.read" /></template>
