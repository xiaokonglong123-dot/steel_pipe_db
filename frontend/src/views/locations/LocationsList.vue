<script setup lang="ts">
import CrudList from "@/components/CrudList.vue"
const columns = [{ prop: "warehouse_id", label: "仓库 ID", required: true }, { prop: "code", label: "编码" }, { prop: "name", label: "库位名称", required: true }, { prop: "status", label: "状态" }] as const
</script>
<template><CrudList title="库位" endpoint="/locations" :columns="columns" permission="stock.read" /></template>
