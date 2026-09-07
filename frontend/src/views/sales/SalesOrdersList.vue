<script setup lang="ts">
import { onMounted, reactive, ref } from "vue"
import { useRouter } from "vue-router"
import { ElMessage } from "element-plus"
import { DataTable, PageHeader, SearchBar, EntitySelect, OrderStatusTag, MoneyText } from "@/components"
import { listSalesOrders } from "@/api/sales"
import { searchCustomers } from "@/api/selects"
import { useHasPermission } from "@/composables/useHasPermission"
import type { SalesOrder } from "@/types/order"

const router = useRouter()
const can = useHasPermission()

const rows = ref<SalesOrder[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const loading = ref(false)
const filters = reactive<{ order_no: string; status: string; customer_id: number | null }>({
  order_no: "", status: "", customer_id: null,
})

const statusOptions = [
  { value: "draft", label: "草稿" },
  { value: "submitted", label: "待审批" },
  { value: "approved", label: "已审批" },
  { value: "rejected", label: "已驳回" },
  { value: "cancelled", label: "已取消" },
  { value: "shipped", label: "已发货" },
]

async function load(): Promise<void> {
  loading.value = true
  try {
    const r = await listSalesOrders({
      page: page.value,
      page_size: pageSize.value,
      ...(filters.order_no ? { order_no: filters.order_no } : {}),
      ...(filters.status ? { status: filters.status } : {}),
      ...(filters.customer_id != null ? { customer_id: filters.customer_id } : {}),
    })
    rows.value = [...r.items]
    total.value = r.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

function reset(): void {
  filters.order_no = ""
  filters.status = ""
  filters.customer_id = null
  page.value = 1
  void load()
}

onMounted(() => void load())
</script>

<template>
  <section class="page">
    <PageHeader title="销售订单" subtitle="录单 → ATP 校验 → 审批 → 发货">
      <el-button v-if="can('order.write')" type="primary" @click="router.push('/sales-orders/new')">新建销售单</el-button>
    </PageHeader>

    <el-card>
      <SearchBar :model-value="filters" @search="page = 1; load()" @reset="reset">
        <el-form-item label="订单号"><el-input v-model="filters.order_no" clearable /></el-form-item>
        <el-form-item label="客户">
          <EntitySelect v-model="filters.customer_id" :api="searchCustomers" placeholder="搜索客户" width="220px" />
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="filters.status" clearable style="width: 140px">
            <el-option v-for="o in statusOptions" :key="o.value" :value="o.value" :label="o.label" />
          </el-select>
        </el-form-item>
      </SearchBar>

      <DataTable
        :columns="[
          { prop: 'order_no', label: '订单号' },
          { prop: 'contract_no', label: '合同号' },
          { prop: 'customer_name', label: '客户' },
          { prop: 'order_date', label: '订单日期' },
          { prop: 'status', label: '状态' },
          { prop: 'total_amount', label: '总金额' },
        ]"
        :data="rows"
        :total="total"
        :page="page"
        :page-size="pageSize"
        :loading="loading"
        @page-change="page = $event; load()"
        @page-size-change="pageSize = $event; page = 1; load()"
      >
        <template #actions="{ row }">
          <el-button link type="primary" @click="router.push(`/sales-orders/${(row as SalesOrder).id}`)">查看</el-button>
        </template>
        <template #cell-status="{ row }">
          <OrderStatusTag :status="(row as SalesOrder).status" variant="sales" />
        </template>
        <template #cell-total_amount="{ row }">
          <MoneyText :value="(row as SalesOrder).total_amount" strong />
        </template>
      </DataTable>
    </el-card>
  </section>
</template>