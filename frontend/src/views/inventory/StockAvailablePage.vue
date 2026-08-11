<script setup lang="ts">
import { ref, onMounted } from "vue"
import { ElMessage } from "element-plus"
import { get } from "@/api/client"
import type { AvailableQty } from "@/types"

const itemId = ref<number>(0)
const locationId = ref<number | null>(null)
const result = ref<AvailableQty | null>(null)
const loading = ref(false)

async function query(): Promise<void> {
  if (!itemId.value) { ElMessage.warning("请输入商品 ID"); return }
  loading.value = true
  try {
    const qs = new URLSearchParams({ item_id: String(itemId.value) })
    if (locationId.value) qs.set("location_id", String(locationId.value))
    const r = await get<AvailableQty>(`/inventory/available?${qs.toString()}`)
    result.value = r
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "查询失败")
  } finally { loading.value = false }
}

onMounted(() => {})
</script>

<template>
  <section class="page">
    <div class="heading"><h2>库存查询 + ATP 可用量</h2></div>
    <el-card>
      <el-form inline>
        <el-form-item label="商品 ID"><el-input-number v-model="itemId" :min="0" controls-position="right" /></el-form-item>
        <el-form-item label="库位 ID（可选）"><el-input-number v-model="locationId" :min="0" controls-position="right" /></el-form-item>
        <el-form-item><el-button type="primary" :loading="loading" @click="query">查询</el-button></el-form-item>
      </el-form>
      <el-descriptions v-if="result" :column="3" border>
        <el-descriptions-item label="商品 ID">{{ result.item_id }}</el-descriptions-item>
        <el-descriptions-item label="库位 ID">{{ result.location_id ?? "全部" }}</el-descriptions-item>
        <el-descriptions-item label="可用量">{{ result.available_qty }}</el-descriptions-item>
      </el-descriptions>
    </el-card>
  </section>
</template>

<style scoped>
.heading { margin-bottom: 16px; }
.heading h2 { margin: 0; }
</style>
