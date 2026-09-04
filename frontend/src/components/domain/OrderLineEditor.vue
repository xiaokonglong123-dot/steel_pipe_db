<script setup lang="ts">
import { computed } from "vue"
import { ElMessage } from "element-plus"
import EntitySelect, { type SelectApi } from "./EntitySelect.vue"
import { add, formatQty, mul } from "@/utils/money"

export interface LineField {
  item_id: number | null
  quantity: string
  unit_price?: string
  notes?: string
}

const props = withDefaults(
  defineProps<{
    lines: LineField[]
    itemApi: SelectApi
    showPrice?: boolean
    currency?: string
  }>(),
  { showPrice: true, currency: "CNY" },
)

const emit = defineEmits<{ (e: "update:lines", lines: LineField[]): void }>()

function notify(): void {
  emit("update:lines", props.lines)
}

function addLine(): void {
  props.lines.push({ item_id: null, quantity: "1", unit_price: "0" })
  notify()
}

function removeLine(index: number): void {
  props.lines.splice(index, 1)
  notify()
}

function lineSubtotal(line: LineField): string {
  if (!props.showPrice) return ""
  return mul(line.quantity || "0", line.unit_price || "0")
}

const total = computed(() => {
  if (!props.showPrice) return ""
  return add(...props.lines.map((l) => mul(l.quantity || "0", l.unit_price || "0")))
})

const hasData = computed(() =>
  props.lines.some((l) => l.item_id != null && formatQty(l.quantity) !== "0"),
)

function validate(): boolean {
  if (props.lines.length === 0) {
    ElMessage.warning("请至少添加一行明细")
    return false
  }
  for (const [i, l] of props.lines.entries()) {
    if (l.item_id == null) {
      ElMessage.warning(`第 ${i + 1} 行未选择商品`)
      return false
    }
    if (formatQty(l.quantity) === "0") {
      ElMessage.warning(`第 ${i + 1} 行数量必须大于 0`)
      return false
    }
    if (props.showPrice && Number(l.unit_price ?? "0") < 0) {
      ElMessage.warning(`第 ${i + 1} 行单价不能为负`)
      return false
    }
  }
  return true
}

defineExpose({ validate, hasData })

function onQty(idx: number, v?: number): void {
  const line = props.lines[idx]
  if (!line) return
  line.quantity = String(v ?? 0)
  notify()
}
function onPrice(idx: number, v?: number): void {
  const line = props.lines[idx]
  if (!line) return
  line.unit_price = String(v ?? 0)
  notify()
}
</script>

<template>
  <div class="line-editor">
    <el-table :data="lines" border size="small">
      <el-table-column label="#" type="index" width="46" />
      <el-table-column label="商品" min-width="240">
        <template #default="{ row }">
          <EntitySelect
            :model-value="row.item_id"
            :api="itemApi"
            placeholder="搜索并选择商品"
            @update:model-value="(v: number | null) => { row.item_id = v; notify() }"
          />
        </template>
      </el-table-column>
      <el-table-column label="数量" width="150">
        <template #default="{ row, $index }">
          <el-input-number
            :model-value="Number(row.quantity ?? 0)"
            :min="0"
            :precision="4"
            :controls="false"
            style="width: 100%"
            @update:model-value="(v?: number) => onQty($index, v)"
          />
        </template>
      </el-table-column>
      <el-table-column v-if="showPrice" label="单价" width="160">
        <template #default="{ row, $index }">
          <el-input-number
            :model-value="Number(row.unit_price ?? 0)"
            :min="0"
            :precision="2"
            :controls="false"
            style="width: 100%"
            @update:model-value="(v?: number) => onPrice($index, v)"
          />
        </template>
      </el-table-column>
      <el-table-column v-if="showPrice" label="行小计" width="150" align="right">
        <template #default="{ row }">{{ formatQty(lineSubtotal(row)) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="70" align="center">
        <template #default="{ $index }">
          <el-button link type="danger" @click="removeLine($index)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <div class="line-toolbar">
      <el-button size="small" @click="addLine">+ 添加明细行</el-button>
      <div v-if="showPrice" class="total">
        合计：<strong>{{ formatQty(total) }}</strong>
        <span class="currency">{{ currency }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.line-editor { width: 100%; }
.line-toolbar { display: flex; justify-content: space-between; align-items: center; margin-top: 12px; }
.total { font-size: 15px; }
.currency { margin-left: 4px; color: var(--muted); }
</style>