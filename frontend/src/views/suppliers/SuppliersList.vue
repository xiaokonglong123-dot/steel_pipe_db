<script setup lang="ts">
import CrudList from "@/components/CrudList.vue"
const columns = [{ prop: "code", label: "编码" }, { prop: "name", label: "名称", required: true }, { prop: "contact", label: "联系人" }, { prop: "phone", label: "电话" }, { prop: "status", label: "状态" }] as const
</script>
<template><CrudList title="供应商" endpoint="/suppliers" :columns="columns" :search-fields="columns.slice(0, 2)" permission="supplier.read" /></template>
