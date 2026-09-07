<script setup lang="ts">
import { computed, ref } from "vue"
import { useRouter } from "vue-router"
import { ElMessage } from "element-plus"
import { EntitySelect, PageHeader } from "@/components"
import { searchItems, searchLocations, searchSuppliers, searchCustomers } from "@/api/selects"
import { createInbound, createOutbound } from "@/api/inventory"

const props = defineProps<{ mode: "inbound" | "outbound" }>()
const router = useRouter()

interface Line {
  item_id: number | null
  location_id: number | null
  quantity: string
}

const isInbound = props.mode === "inbound"

const INBOUND_TYPES = [
  { value: "purchase", label: "采购入库" },
  { value: "production", label: "生产入库" },
  { value: "return", label: "退货入库" },
  { value: "other", label: "其他" },
]
const OUTBOUND_TYPES = [
  { value: "sales", label: "销售出库" },
  { value: "requisition", label: "领用出库" },
  { value: "other", label: "其他" },
]

const typeOptions = computed(() => (isInbound ? INBOUND_TYPES : OUTBOUND_TYPES))
const typeValue = ref(isInbound ? "purchase" : "sales")
const typeCaption = isInbound ? "入库类型" : "出库类型"

const partyId = ref<number | null>(null)
const notes = ref("")
const lines = ref<Line[]>([{ item_id: null, location_id: null, quantity: "1" }])
const saving = ref(false)

const partyApi = isInbound ? searchSuppliers : searchCustomers
const partyLabel = isInbound ? "供应商（可选）" : "客户（可选）"

function addLine(): void {
  lines.value.push({ item_id: null, location_id: null, quantity: "1" })
}

function removeLine(index: number): void {
  lines.value.splice(index, 1)
}

function onQty(index: number, v?: number): void {
  const line = lines.value[index]
  if (line) line.quantity = String(v ?? 0)
}

function validQuantity(q: string): boolean {
  const n = Number(q)
  return Number.isFinite(n) && n > 0
}

async function save(): Promise<void> {
  const items: { item_id: number; location_id: number; quantity: string }[] = []
  for (const [i, l] of lines.value.entries()) {
    if (l.item_id == null || l.location_id == null) {
      ElMessage.warning(`第 ${i + 1} 行需选择商品和库位`)
      return
    }
    if (!validQuantity(l.quantity)) {
      ElMessage.warning(`第 ${i + 1} 行数量必须大于 0`)
      return
    }
    items.push({ item_id: l.item_id, location_id: l.location_id, quantity: l.quantity })
  }
  if (items.length === 0) {
    ElMessage.warning("请至少添加一行明细")
    return
  }
  saving.value = true
  try {
    if (isInbound) {
      const d = await createInbound({
        inbound_type: typeValue.value,
        ...(partyId.value != null ? { supplier_id: partyId.value } : {}),
        ...(notes.value ? { notes: notes.value } : {}),
        items,
      })
      void router.push(`/inventory/inbound/${d.id}`)
    } else {
      const d = await createOutbound({
        outbound_type: typeValue.value,
        ...(partyId.value != null ? { customer_id: partyId.value } : {}),
        ...(notes.value ? { notes: notes.value } : {}),
        items,
      })
      void router.push(`/inventory/outbound/${d.id}`)
    }
    ElMessage.success("已创建，可过账")
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "保存失败")
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <section class="page">
    <PageHeader :title="isInbound ? '新建入库单' : '新建出库单'" />
    <el-card>
      <el-form label-width="120px">
        <el-form-item :label="typeCaption" required>
          <el-select v-model="typeValue" style="width: 200px">
            <el-option v-for="o in typeOptions" :key="o.value" :value="o.value" :label="o.label" />
          </el-select>
        </el-form-item>
        <el-form-item :label="partyLabel">
          <EntitySelect v-model="partyId" :api="partyApi" :placeholder="partyLabel" width="320px" />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="notes" type="textarea" :rows="2" />
        </el-form-item>
      </el-form>
    </el-card>

    <el-card class="lines">
      <template #header>明细（商品 + 库位 + 数量）</template>
      <el-table :data="lines" border size="small">
        <el-table-column type="index" label="#" width="46" />
        <el-table-column label="商品" min-width="240">
          <template #default="{ row }">
            <EntitySelect v-model="row.item_id" :api="searchItems" placeholder="搜索并选择商品" />
          </template>
        </el-table-column>
        <el-table-column label="库位" min-width="220">
          <template #default="{ row }">
            <EntitySelect v-model="row.location_id" :api="searchLocations" placeholder="搜索并选择库位" />
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
        <el-table-column label="操作" width="70" align="center">
          <template #default="{ $index }">
            <el-button link type="danger" @click="removeLine($index)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="add-row"><el-button size="small" @click="addLine">+ 添加明细行</el-button></div>
    </el-card>

    <div class="footer">
      <el-button @click="router.back()">返回</el-button>
      <el-button type="primary" :loading="saving" @click="save">保存</el-button>
    </div>
  </section>
</template>

<style scoped>
.lines { margin-top: 16px; }
.add-row { margin-top: 12px; }
.footer { margin-top: 16px; display: flex; justify-content: flex-end; gap: 8px; }
</style>