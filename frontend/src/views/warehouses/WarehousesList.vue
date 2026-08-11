<script setup lang="ts">
import CrudList from "@/components/CrudList.vue"
const columns = [{ prop: "code", label: "编码" }, { prop: "name", label: "名称", required: true }, { prop: "address", label: "地址" }, { prop: "status", label: "状态" }] as const
</script>
<template><CrudList title="仓库" endpoint="/warehouses" :columns="columns" permission="stock.read" /></template>
