<script setup lang="ts">
import { onMounted, reactive, ref } from "vue"
import { ElMessage } from "element-plus"
import { DataTable, EntitySelect, MoneyText, PageHeader } from "@/components"
import { listInvoices, createInvoice } from "@/api/finance"
import { searchSuppliers, searchCustomers } from "@/api/selects"
import { useHasPermission } from "@/composables/useHasPermission"
import type { Invoice, InvoicePayload } from "@/types/finance"

const can = useHasPermission()

const rows = ref<Invoice[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const loading = ref(false)

const dialog = ref(false)
const saving = ref(false)
const form = reactive<{ invoice_no: string; invoice_date: string; party_type: "supplier" | "customer"; party_id: number | null; amount: string }>({
  invoice_no: "", invoice_date: new Date().toISOString().slice(0, 10), party_type: "supplier", party_id: null, amount: "0",
})

const partyApi = () => (form.party_type === "supplier" ? searchSuppliers : searchCustomers)

function partyTypeLabel(t: string): string {
  return t === "supplier" ? "供应商" : "客户"
}

async function load(): Promise<void> {
  loading.value = true
  try {
    const r = await listInvoices()
    rows.value = [...r.items]
    total.value = r.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

function openCreate(): void {
  form.invoice_no = ""
  form.invoice_date = new Date().toISOString().slice(0, 10)
  form.party_type = "supplier"
  form.party_id = null
  form.amount = "0"
  dialog.value = true
}

async function save(): Promise<void> {
  if (!form.invoice_no.trim() || form.party_id == null) {
    ElMessage.warning("请填写发票号并选择往来方")
    return
  }
  saving.value = true
  try {
    const payload: InvoicePayload = {
      invoice_no: form.invoice_no,
      invoice_date: form.invoice_date,
      party_type: form.party_type,
      party_id: form.party_id,
      amount: form.amount || "0",
    }
    await createInvoice(payload)
    dialog.value = false
    ElMessage.success("发票已创建")
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
    <PageHeader title="发票" subtitle="应收/应付发票登记">
      <el-button v-if="can('finance.write')" type="primary" @click="openCreate">新建发票</el-button>
    </PageHeader>

    <el-card>
      <DataTable
        :columns="[
          { prop: 'invoice_no', label: '发票号' },
          { prop: 'invoice_date', label: '日期' },
          { prop: 'party_type', label: '往来方类型' },
          { prop: 'party_id', label: '往来方' },
          { prop: 'amount', label: '金额' },
          { prop: 'status', label: '状态' },
        ]"
        :data="rows"
        :total="total"
        :page="page"
        :page-size="pageSize"
        :loading="loading"
        @page-change="page = $event; load()"
        @page-size-change="pageSize = $event; page = 1; load()"
      >
        <template #cell-party_type="{ row }">{{ partyTypeLabel((row as Invoice).party_type) }}</template>
        <template #cell-amount="{ row }"><MoneyText :value="(row as Invoice).amount" strong /></template>
      </DataTable>
    </el-card>

    <el-dialog v-model="dialog" title="新建发票" width="520px" destroy-on-close>
      <el-form label-width="100px">
        <el-form-item label="发票号" required><el-input v-model="form.invoice_no" /></el-form-item>
        <el-form-item label="日期">
          <el-date-picker v-model="form.invoice_date" type="date" value-format="YYYY-MM-DD" />
        </el-form-item>
        <el-form-item label="往来方类型">
          <el-select v-model="form.party_type" style="width: 100%">
            <el-option value="supplier" label="供应商" />
            <el-option value="customer" label="客户" />
          </el-select>
        </el-form-item>
        <el-form-item label="往来方" required>
          <EntitySelect v-model="form.party_id" :api="partyApi()" placeholder="搜索并选择" />
        </el-form-item>
        <el-form-item label="金额" required>
          <el-input v-model="form.amount" placeholder="如 1200.50" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialog = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="save">保存</el-button>
      </template>
    </el-dialog>
  </section>
</template>