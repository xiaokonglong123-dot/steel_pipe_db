<script setup lang="ts">
defineProps<{ readonly modelValue: Record<string, string> }>()
const emit = defineEmits<{ (event: "search"): void; (event: "reset"): void }>()
</script>

<template>
  <el-form inline @submit.prevent="emit('search')"><slot /><el-form-item><el-button type="primary" @click="emit('search')">查询</el-button><el-button @click="emit('reset')">重置</el-button></el-form-item></el-form>
</template>
