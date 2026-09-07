<script setup lang="ts">
import { onMounted, ref } from "vue"

export interface Selectable {
  readonly id: number
  readonly name: string
  readonly code?: string | null
}
export type SelectApi = (query: string) => Promise<readonly Selectable[]>

const props = withDefaults(
  defineProps<{
    modelValue?: number | null
    api: SelectApi
    placeholder?: string
    clearable?: boolean
    disabled?: boolean
    width?: string
  }>(),
  { modelValue: null, placeholder: "请选择", clearable: true, disabled: false, width: "100%" },
)

const emit = defineEmits<{ (e: "update:modelValue", v: number | null): void }>()

const options = ref<readonly Selectable[]>([])
const loading = ref(false)
const searching = ref(false)
let timer: ReturnType<typeof setTimeout> | undefined
let fired = false

async function load(query: string): Promise<void> {
  loading.value = true
  try {
    options.value = await props.api(query)
  } finally {
    loading.value = false
  }
}

function onSearch(query: string): void {
  if (timer) clearTimeout(timer)
  timer = setTimeout(() => void load(query), 250)
}

function onChange(v: number | string | undefined): void {
  emit("update:modelValue", typeof v === "number" ? v : null)
}

onMounted(() => {
  if (!fired) {
    fired = true
    void load("")
  }
})
</script>

<template>
  <el-select
    :model-value="modelValue ?? undefined"
    :placeholder="placeholder"
    :clearable="clearable"
    :disabled="disabled"
    filterable
    remote
    :remote-method="onSearch"
    :loading="loading"
    :style="{ width }"
    @update:model-value="onChange"
    @focus="searching = !searching"
  >
    <el-option
      v-for="opt in options"
      :key="opt.id"
      :value="opt.id"
      :label="opt.code ? `${opt.name} (${opt.code})` : opt.name"
    />
  </el-select>
</template>