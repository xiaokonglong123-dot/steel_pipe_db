<script setup lang="ts">
const props = defineProps<{ readonly status?: string }>()
const colors: Record<string, "success" | "warning" | "danger" | "info"> = { active: "success", enabled: "success", approved: "success", completed: "success", pending: "warning", draft: "info", rejected: "danger", cancelled: "danger", disabled: "danger" }
const labels: Record<string, string> = { active: "启用", enabled: "启用", approved: "已通过", completed: "已完成", pending: "待处理", draft: "草稿", rejected: "已拒绝", cancelled: "已取消", disabled: "停用" }
</script>
<template><el-tag :type="colors[props.status ?? ''] ?? 'info'">{{ labels[props.status ?? ''] ?? props.status ?? "未知" }}</el-tag></template>
