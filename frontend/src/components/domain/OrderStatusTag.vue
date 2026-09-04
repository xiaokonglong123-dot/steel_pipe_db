<script setup lang="ts">
import { computed } from "vue"
import { PURCHASE_STATUS, SALES_STATUS } from "@/utils/actions"

const props = withDefaults(
  defineProps<{
    status: string
    variant?: "purchase" | "sales" | "instance" | "task" | "generic"
  }>(),
  { variant: "generic" },
)

const mapping = computed(() => {
  switch (props.variant) {
    case "purchase":
      return PURCHASE_STATUS
    case "sales":
      return SALES_STATUS
    default:
      return {}
  }
})

// 兜底：未命中映射也给出一个中性标签
const meta = computed(() => mapping.value[props.status] ?? { label: props.status, type: "" })
</script>

<template>
  <el-tag :type="meta.type" size="small">{{ meta.label }}</el-tag>
</template>