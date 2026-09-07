<script setup lang="ts">
import { ref } from "vue"
import { ElMessage } from "element-plus"
import { EntitySelect, PageHeader } from "@/components"
import { getAvailable } from "@/api/inventory"
import { searchItems, searchLocations } from "@/api/selects"
import type { AvailableQty } from "@/types/inventory"

const itemId = ref<number | null>(null)
const locationId = ref<number | null>(null)
const result = ref<AvailableQty | null>(null)
const loading = ref(false)

async function query(): Promise<void> {
  if (itemId.value == null) {
    ElMessage.warning("请选择商品")
    return
  }
  loading.value = true
  try {
    result.value = await getAvailable(itemId.value, locationId.value)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "查询失败")
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <section class="page">
    <PageHeader title="ATP 可用量" subtitle="可用量 = 库存余额 - 已 active 预留" />
    <el-card>
      <el-form inline>
        <el-form-item label="商品">
          <EntitySelect v-model="itemId" :api="searchItems" placeholder="搜索并选择商品" width="260px" />
        </el-form-item>
        <el-form-item label="库位（可选）">
          <EntitySelect v-model="locationId" :api="searchLocations" placeholder="全部库位" width="220px" />
        </el-form-item>
        <el-form-item><el-button type="primary" :loading="loading" @click="query">查询</el-button></el-form-item>
      </el-form>

      <el-descriptions v-if="result" :column="3" border>
        <el-descriptions-item label="商品 ID">{{ result.item_id }}</el-descriptions-item>
        <el-descriptions-item label="库位">{{ result.location_id ?? "全部" }}</el-descriptions-item>
        <el-descriptions-item label="可用量">
          <strong>{{ result.available_qty }}</strong>
        </el-descriptions-item>
      </el-descriptions>
    </el-card>
  </section>
</template>