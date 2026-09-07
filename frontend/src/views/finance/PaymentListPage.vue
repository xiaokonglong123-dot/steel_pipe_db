<script setup lang="ts">
import { onMounted, reactive, ref } from "vue"
import { ElMessage } from "element-plus"
import { DataTable, EntitySelect, MoneyText, PageHeader } from "@/components"
import { listPayments, createPayment } from "@/api/finance"
import { searchSuppliers } from "@/api/selects"
import { useHasPermission } from "@/composables/useHasPermission"
import type { Payment, PaymentPayload } from "@/types/finance"

const can = useHasPermission()

const rows = ref<Payment[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const loading = ref(false)

const dialog = ref(false)
const saving = ref(false)
const form = reactive<{ payment_no: string; payment_date: string; supplier_id: number | null; amount: string; method: string; notes: string }>({
  payment_no: "", payment_date: new Date().toISOString().slice(0, 10), supplier_id: null, amount: "0", method: "", notes: "",
})

async function load(): Promise<void> {
  loading.value = true
  try {
    const r = await listPayments()
    rows.value = [...r.items]
    total.value = r.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

function openCreate(): void {
  form.payment_no = ""
  form.payment_date = new Date().toISOString().slice(0, 10)
  form.supplier_id = null
  form.amount = "0"
  form.method = ""
  form.notes = ""
  dialog.value = true
}

async function save(): Promise<void> {
  if (!form.payment_no.trim()) {
    ElMessage.warning("请填写付款单号")
    return
  }
  saving.value = true
  try {
    const payload: PaymentPayload = {
      payment_no: form.payment_no,
      payment_date: form.payment_date,
      ...(form.supplier_id != null ? { supplier_id: form.supplier_id } : {}),
      amount: form.amount || "0",
      ...(form.method ? { method: form.method } : {}),
      ...(form.notes ? { notes: form.notes } : {}),
    }
    await createPayment(payload)
    dialog.value = false
    ElMessage.success("付款已登记")
    void load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "保存失败")
  } finally {
    saving.value = false
  }
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <PageHeader title="付款" subtitle="付款登记">
      <el-button v-if="can('finance.write')" type="primary" @click="openCreate">新建付款</el-button>
    </PageHeader>

    <el-card>
      <DataTable
        :columns="[
          { prop: 'payment_no', label: '付款单号' },
          { prop: 'payment_date', label: '日期' },
          { prop: 'supplier_id', label: '供应商' },
          { prop: 'amount', label: '金额' },
          { prop: 'method', label: '方式' },
        ]"
        :data="rows"
        :total="total"
        :page="page"
        :page-size="pageSize"
        :loading="loading"
        @page-change="page = $event; load()"
        @page-size-change="pageSize = $event; page = 1; load()"
      >
        <template #cell-amount="{ row }"><MoneyText :value="(row as Payment).amount" strong /></template>
      </DataTable>
    </el-card>

    <el-dialog v-model="dialog" title="新建付款" width="520px" destroy-on-close>
      <el-form label-width="100px">
        <el-form-item label="付款单号" required><el-input v-model="form.payment_no" /></el-form-item>
        <el-form-item label="日期">
          <el-date-picker v-model="form.payment_date" type="date" value-format="YYYY-MM-DD" />
        </el-form-item>
        <el-form-item label="供应商">
          <EntitySelect v-model="form.supplier_id" :api="searchSuppliers" placeholder="搜索并选择供应商" />
        </el-form-item>
        <el-form-item label="金额" required>
          <el-input v-model="form.amount" placeholder="如 5000.00" />
        </el-form-item>
        <el-form-item label="付款方式">
          <el-select v-model="form.method" clearable style="width: 100%">
            <el-option value="bank" label="银行转账" />
            <el-option value="cash" label="现金" />
            <el-option value="other" label="其他" />
          </el-select>
        </el-form-item>
        <el-form-item label="备注"><el-input v-model="form.notes" type="textarea" :rows="2" /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialog = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="save">保存</el-button>
      </template>
    </el-dialog>
  </section>
</template>