<script setup lang="ts">
import { ref, onMounted, watch } from "vue"
import { ElMessage } from "element-plus"
import { get, post } from "@/api/client"
import type { Invoice, Party, CreateInvoiceRequest, ApiPage } from "@/types"

const rows = ref<readonly Invoice[]>([])
const parties = ref<readonly Party[]>([])
const total = ref(0); const page = ref(1); const pageSize = ref(20); const loading = ref(false)
const dialog = ref(false)
const form = ref<CreateInvoiceRequest>({
  invoice_no: "", invoice_date: new Date().toISOString().slice(0, 10),
  party_type: "supplier", party_id: 0, amount: "0",
})

async function loadParties(): Promise<void> {
  try {
    const result = await get<readonly Party[] | ApiPage<Party>>("/customers")
    parties.value = isPage(result) ? result.items : result
  } catch { /* ignore */ }
}
function isPage<T>(r: readonly T[] | ApiPage<T>): r is ApiPage<T> { return Array.isArray((r as ApiPage<T>).items) }

async function load(): Promise<void> {
  loading.value = true
  try {
    const result = await get<ApiPage<Invoice>>(`/invoices?page=${page.value}&page_size=${pageSize.value}`)
    rows.value = result.items; total.value = result.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally { loading.value = false }
}

function statusLabel(s: Invoice["status"]): string {
  if (s === "paid") return "已付清"
  if (s === "partially_paid") return "部分付款"
  return "未付款"
}
function statusType(s: Invoice["status"]): "danger" | "warning" | "success" {
  if (s === "paid") return "success"
  if (s === "partially_paid") return "warning"
  return "danger"
}

async function save(): Promise<void> {
  try {
    await post("/invoices", form.value)
    dialog.value = false
    ElMessage.success("创建成功")
    await load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "保存失败")
  }
}

onMounted(() => { void load(); void loadParties() })
watch([page, pageSize], () => void load())
</script>

<template>
  <section class="page">
    <div class="heading"><h2>发票</h2><el-button type="primary" @click="dialog = true">新建发票</el-button></div>
    <el-card>
      <el-table :data="rows" v-loading="loading" border>
        <el-table-column prop="invoice_no" label="发票号" width="160" />
        <el-table-column prop="invoice_date" label="日期" width="120" />
        <el-table-column prop="party_type" label="往来方类型" width="100" />
        <el-table-column prop="party_id" label="往来方 ID" width="100" />
        <el-table-column prop="amount" label="金额" />
        <el-table-column label="状态" width="120">
          <template #default="{ row }">
            <el-tag :type="statusType((row as Invoice).status)">{{ statusLabel((row as Invoice).status) }}</el-tag>
          </template>
        </el-table-column>
      </el-table>
      <el-pagination :current-page="page" :page-size="pageSize" :total="total" layout="total, prev, pager, next" @current-change="page = $event" />
    </el-card>
    <el-dialog v-model="dialog" title="新建发票" width="500px">
      <el-form :model="form" label-width="100px">
        <el-form-item label="发票号"><el-input v-model="form.invoice_no" /></el-form-item>
        <el-form-item label="日期"><el-date-picker v-model="form.invoice_date" type="date" value-format="YYYY-MM-DD" /></el-form-item>
        <el-form-item label="往来方类型">
          <el-select v-model="form.party_type">
            <el-option value="supplier" label="供应商" />
            <el-option value="customer" label="客户" />
          </el-select>
        </el-form-item>
        <el-form-item label="往来方"><el-input-number v-model="form.party_id" :min="0" controls-position="right" /></el-form-item>
        <el-form-item label="金额"><el-input v-model="form.amount" /></el-form-item>
      </el-form>
      <template #footer><el-button @click="dialog = false">取消</el-button><el-button type="primary" @click="save">保存</el-button></template>
    </el-dialog>
  </section>
</template>

<style scoped>
.heading { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.heading h2 { margin: 0; }
</style>
