<script setup lang="ts">
import CrudList from "@/components/CrudList.vue"
const columns = [{ prop: "item_id", label: "商品 ID" }, { prop: "location_id", label: "库位 ID" }, { prop: "quantity", label: "库存数量" }, { prop: "available_quantity", label: "可用数量" }] as const
</script>
<template><CrudList title="库存查询" endpoint="/inventory" :columns="columns" :search-fields="columns.slice(0, 2)" permission="stock.read" /></template>
