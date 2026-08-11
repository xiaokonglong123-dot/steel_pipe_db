<script setup lang="ts">
import { useHasPermission } from "@/composables/useHasPermission"
const props = defineProps<{ readonly permission: string }>()
const allowed = useHasPermission()
</script>
<template><el-button v-if="allowed(props.permission)"><slot /></el-button></template>
