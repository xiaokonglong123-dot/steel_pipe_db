<script setup lang="ts">
import { ACTION_LABELS, ACTION_TYPES } from "@/utils/actions"
import type { OrderAction } from "@/types/order"

// 仅渲染由后端 allowed_actions 驱动的动作按钮（ADR-R4）。
// 按钮点击发出 action 事件，具体业务逻辑由父组件处理，并在父组件做权限二次校验。
const props = defineProps<{ actions: readonly string[] }>()
const emit = defineEmits<{ (e: "action", action: OrderAction): void }>()

function labelOf(a: string): string {
  return (ACTION_LABELS as Record<string, string>)[a] ?? a
}
function typeOf(a: string): string {
  return (ACTION_TYPES as Record<string, string>)[a] ?? "primary"
}
function onClick(a: string): void {
  emit("action", a as OrderAction)
}
</script>

<template>
  <span class="action-bar">
    <el-button
      v-for="a in actions"
      :key="a"
      :type="typeOf(a) === 'danger' ? 'danger' : typeOf(a) === 'success' ? 'success' : typeOf(a) === 'warning' ? 'warning' : 'primary'"
      size="small"
      plain
      @click="onClick(a)"
    >
      {{ labelOf(a) }}
    </el-button>
  </span>
</template>

<style scoped>
.action-bar { display: inline-flex; gap: 8px; flex-wrap: wrap; }
</style>