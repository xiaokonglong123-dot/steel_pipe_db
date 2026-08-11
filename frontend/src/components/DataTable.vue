<script setup lang="ts" generic="T extends Record<string, unknown>">
import type { PropType } from "vue"

type Column = { readonly prop: string; readonly label: string; readonly width?: number }
defineProps({
  columns: { type: Array as PropType<readonly Column[]>, required: true },
  data: { type: Array as PropType<readonly T[]>, required: true },
  total: { type: Number, required: true },
  loading: { type: Boolean, default: false },
  page: { type: Number, default: 1 },
  pageSize: { type: Number, default: 20 },
})
const emit = defineEmits<{ (event: "page-change", page: number): void; (event: "page-size-change", size: number): void }>()
</script>

<template>
  <el-table :data="data" v-loading="loading" border stripe>
    <el-table-column v-for="column in columns" :key="column.prop" v-bind="column" :prop="column.prop" :label="column.label" />
    <el-table-column label="操作" width="180" fixed="right"><template #default="scope"><slot name="actions" :row="scope.row" /></template></el-table-column>
  </el-table>
  <el-pagination class="table-pagination" background layout="total, sizes, prev, pager, next" :total="total" :current-page="page" :page-size="pageSize" :page-sizes="[10, 20, 50]" @current-change="emit('page-change', $event)" @size-change="emit('page-size-change', $event)" />
</template>

<style scoped>
.table-pagination { margin-top: 16px; justify-content: flex-end; }
</style>
