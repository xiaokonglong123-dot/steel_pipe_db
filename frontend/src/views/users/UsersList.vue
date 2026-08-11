<script setup lang="ts">
import CrudList from "@/components/CrudList.vue"
const columns = [{ prop: "username", label: "用户名", required: true }, { prop: "display_name", label: "显示名称", required: true }, { prop: "email", label: "邮箱" }, { prop: "status", label: "状态" }] as const
</script>
<template><CrudList title="用户管理" endpoint="/users" :columns="columns" permission="user.manage" /></template>
