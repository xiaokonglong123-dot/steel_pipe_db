<script setup lang="ts">
import CrudList from "@/components/CrudList.vue"
const columns = [{ prop: "id", label: "实例 ID" }, { prop: "business_type", label: "业务类型" }, { prop: "business_id", label: "业务 ID" }, { prop: "status", label: "状态" }, { prop: "created_at", label: "创建时间" }] as const
</script>
<template><CrudList title="流程实例" endpoint="/workflow-instances" :columns="columns" permission="order.approve" /></template>
