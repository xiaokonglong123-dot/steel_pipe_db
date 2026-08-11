<template><div class="loading-overlay"><el-icon class="is-loading"><Loading /></el-icon><span>加载中...</span></div></template>
<script setup lang="ts">
import { Loading } from "@element-plus/icons-vue"
</script>
<style scoped>.loading-overlay { display: flex; align-items: center; justify-content: center; gap: 8px; padding: 32px; color: var(--el-text-color-secondary); }</style>
