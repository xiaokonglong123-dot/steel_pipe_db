<script setup lang="ts">
import { ref, onMounted, watch } from "vue"
import { ElMessage } from "element-plus"
import { get, post } from "@/api/client"
import type { Payment, CreatePaymentRequest, ApiPage } from "@/types"

const rows = ref<readonly Payment[]>([])
const total = ref(0); const page = ref(1); const pageSize = ref(20); const loading = ref(false)
const dialog = ref(false)
const form = ref<CreatePaymentRequest>({
  payment_no: "", payment_date: new Date().toISOString().slice(0, 10),
  supplier_id: null, amount: "0", invoice_id: null, method: null, notes: null,
})

async function load(): Promise<void> {
  loading.value = true
  try {
    const result = await get<ApiPage<Payment>>(`/payments?page=${page.value}&page_size=${pageSize.value}`)
    rows.value = result.items; total.value = result.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally { loading.value = false }
}

async function save(): Promise<void> {
  try {
    await post("/payments", form.value)
    dialog.value = false
    ElMessage.success("创建成功")
    await load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "保存失败")
  }
}

onMounted(() => void load())
watch([page, pageSize], () => void load())
</script>

<template>
  <section class="page">
    <div class="heading"><h2>付款</h2><el-button type="primary" @click="dialog = true">新建付款</el-button></div>
    <el-card>
      <el-table :data="rows" v-loading="loading" border>
        <el-table-column prop="payment_no" label="付款单号" width="160" />
        <el-table-column prop="payment_date" label="日期" width="120" />
        <el-table-column prop="supplier_id" label="供应商 ID" width="120" />
        <el-table-column prop="amount" label="金额" />
        <el-table-column prop="invoice_id" label="关联发票 ID" width="120" />
        <el-table-column prop="method" label="方式" width="120" />
        <el-table-column prop="notes" label="备注" />
      </el-table>
      <el-pagination :current-page="page" :page-size="pageSize" :total="total" layout="total, prev, pager, next" @current-change="page = $event" />
    </el-card>
    <el-dialog v-model="dialog" title="新建付款" width="500px">
      <el-form :model="form" label-width="100px">
        <el-form-item label="付款单号"><el-input v-model="form.payment_no" /></el-form-item>
        <el-form-item label="日期"><el-date-picker v-model="form.payment_date" type="date" value-format="YYYY-MM-DD" /></el-form-item>
        <el-form-item label="供应商"><el-input-number v-model="form.supplier_id" :min="0" controls-position="right" /></el-form-item>
        <el-form-item label="金额"><el-input v-model="form.amount" /></el-form-item>
        <el-form-item label="关联发票"><el-input-number v-model="form.invoice_id" :min="0" controls-position="right" /></el-form-item>
        <el-form-item label="方式">
          <el-select v-model="form.method" clearable>
            <el-option value="bank_transfer" label="银行转账" />
            <el-option value="cash" label="现金" />
            <el-option value="check" label="支票" />
          </el-select>
        </el-form-item>
        <el-form-item label="备注"><el-input v-model="form.notes" type="textarea" /></el-form-item>
      </el-form>
      <template #footer><el-button @click="dialog = false">取消</el-button><el-button type="primary" @click="save">保存</el-button></template>
    </el-dialog>
  </section>
</template>

<style scoped>
.heading { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.heading h2 { margin: 0; }
</style>
