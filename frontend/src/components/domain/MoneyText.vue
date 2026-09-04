<script setup lang="ts">
import { computed } from "vue"
import { formatAmount, formatQty } from "@/utils/money"

const props = withDefaults(
  defineProps<{
    value?: string | number | null
    kind?: "amount" | "qty"
    dp?: number
    strong?: boolean
  }>(),
  { value: null, kind: "amount", dp: 2, strong: false },
)

const text = computed(() =>
  props.kind === "qty" ? formatQty(props.value) : formatAmount(props.value, props.dp),
)
</script>

<template>
  <span :class="strong ? 'money strong' : 'money'">{{ text }}</span>
</template>

<style scoped>
.money { font-variant-numeric: tabular-nums; white-space: nowrap; }
.strong { font-weight: 600; }
</style>