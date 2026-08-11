<script setup lang="ts">
import CrudList from "@/components/CrudList.vue"
const columns = [{ prop: "outbound_no", label: "出库单号" }, { prop: "status", label: "状态" }, { prop: "created_at", label: "创建时间" }] as const
</script>
<template><CrudList title="出库记录" endpoint="/outbounds" :columns="columns" permission="stock.write" /></template>
