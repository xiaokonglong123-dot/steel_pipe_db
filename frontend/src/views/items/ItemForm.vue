<script setup lang="ts">
import CrudList from "@/components/CrudList.vue"
const columns = [{ prop: "sku", label: "SKU", required: true }, { prop: "name", label: "名称", required: true }, { prop: "category", label: "分类" }, { prop: "unit", label: "单位" }, { prop: "specification", label: "规格" }] as const
</script>
<template><CrudList title="商品编辑" endpoint="/items" :columns="columns" permission="item.write" /></template>
