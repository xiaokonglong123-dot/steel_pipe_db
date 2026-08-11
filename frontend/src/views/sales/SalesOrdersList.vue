<script setup lang="ts">
import CrudList from "@/components/CrudList.vue"
const columns = [{ prop: "order_no", label: "订单号" }, { prop: "customer_id", label: "客户 ID" }, { prop: "status", label: "状态" }, { prop: "expected_date", label: "预计日期" }, { prop: "total_amount", label: "总金额" }] as const
</script>
<template><CrudList title="销售订单" endpoint="/sales-orders" :columns="columns" :search-fields="[columns[2]]" permission="order.read" /></template>
