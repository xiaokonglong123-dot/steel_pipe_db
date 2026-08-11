<script setup lang="ts">
import CrudList from "@/components/CrudList.vue"
const columns = [{ prop: "inbound_no", label: "入库单号" }, { prop: "status", label: "状态" }, { prop: "created_at", label: "创建时间" }] as const
</script>
<template><CrudList title="入库记录" endpoint="/inbounds" :columns="columns" permission="stock.write" /></template>
