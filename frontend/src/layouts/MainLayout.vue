<script setup lang="ts">
import { computed, onMounted } from "vue"
import { useRoute, useRouter } from "vue-router"
import { useAuthStore } from "@/stores/auth"
import { useHasPermission } from "@/composables/useHasPermission"
import { Menu, User, Goods, OfficeBuilding, Box, Document, List, Setting, Wallet, DataAnalysis, Files } from "@element-plus/icons-vue"

const router = useRouter()
const route = useRoute()
const auth = useAuthStore()
const allowed = useHasPermission()


const menuGroups = computed(() => [
  {
    title: "主数据",
    children: [
      { path: "/items", label: "商品", permission: "item.read", icon: Goods },
      { path: "/suppliers", label: "供应商", permission: "supplier.read", icon: OfficeBuilding },
      { path: "/customers", label: "客户", permission: "customer.read", icon: User },
    ],
  },
  {
    title: "库存",
    children: [
      { path: "/warehouses", label: "仓库", permission: "stock.read", icon: Box },
      { path: "/locations", label: "库位", permission: "stock.read", icon: Box },
      { path: "/inventory", label: "库存查询", permission: "stock.read", icon: Box },
      { path: "/inventory/stock-atp", label: "ATP 可用量", permission: "stock.read", icon: Box },
      { path: "/inventory/inbound", label: "入库", permission: "stock.read", icon: Document },
      { path: "/inventory/outbound", label: "出库", permission: "stock.read", icon: Document },
      { path: "/inventory/logs", label: "库存流水", permission: "stock.read", icon: List },
      { path: "/inventory/checks", label: "盘点", permission: "stock.read", icon: Files },
    ],
  },
  {
    title: "业务单据",
    children: [
      { path: "/purchase-orders", label: "采购订单", permission: "order.read", icon: Document },
      { path: "/sales-orders", label: "销售订单", permission: "order.read", icon: Document },
      { path: "/workflow", label: "流程实例", permission: "order.approve", icon: List },
      { path: "/workflow/tasks", label: "待办任务", permission: "order.approve", icon: List },
    ],
  },
  {
    title: "财务",
    children: [
      { path: "/finance/accounts", label: "会计科目", permission: "finance.read", icon: Wallet },
      { path: "/finance/journal-entries", label: "日记账", permission: "finance.read", icon: Wallet },
      { path: "/finance/invoices", label: "发票", permission: "finance.read", icon: Wallet },
      { path: "/finance/payments", label: "付款", permission: "finance.read", icon: Wallet },
      { path: "/finance/trial-balance", label: "试算平衡", permission: "finance.read", icon: Wallet },
    ],
  },
  {
    title: "报表",
    children: [
      { path: "/reports/inventory-summary", label: "库存汇总", permission: "report.read", icon: DataAnalysis },
      { path: "/reports/inbound-outbound", label: "出入库明细", permission: "report.read", icon: DataAnalysis },
      { path: "/reports/sales-trend", label: "销售趋势", permission: "report.read", icon: DataAnalysis },
      { path: "/reports/finance-summary", label: "财务汇总", permission: "report.read", icon: DataAnalysis },
    ],
  },
  {
    title: "系统",
    children: [
      { path: "/users", label: "用户管理", permission: "user.manage", icon: Setting },
      { path: "/operation-logs", label: "操作日志", permission: "report.read", icon: Document },
      { path: "/profile", label: "个人资料", permission: "*", icon: User },
    ],
  },
])

onMounted(() => { void auth.loadMe() })
async function logout(): Promise<void> { await auth.logout(); await router.push("/login") }
</script>

<template>
  <el-container class="app-shell">
    <el-aside width="220px">
      <div class="brand">ERP 管理系统</div>
      <el-menu :default-active="route.path" router>
        <template v-for="group in menuGroups" :key="group.title">
          <el-menu-item-group :title="group.title">
            <el-menu-item v-for="entry in group.children.filter((item) => allowed(item.permission))" :key="entry.path" :index="entry.path">
              <el-icon v-if="entry.icon"><component :is="entry.icon" /></el-icon>
              <span>{{ entry.label }}</span>
            </el-menu-item>
          </el-menu-item-group>
        </template>
      </el-menu>
    </el-aside>
    <el-container>
      <el-header class="header">
        <span>{{ route.meta.title ?? "工作台" }}</span>
        <el-dropdown>
          <span class="user-menu">
            {{ auth.auth_user?.display_name ?? auth.auth_user?.username ?? "用户" }}
            <el-icon><Menu /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item @click="router.push('/profile')">个人资料</el-dropdown-item>
              <el-dropdown-item divided @click="logout">退出登录</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </el-header>
      <el-main class="main"><router-view /></el-main>
    </el-container>
  </el-container>
</template>

<style scoped>
.app-shell { min-height: 100vh; background: var(--surface); }
.brand {
  height: 60px;
  display: flex;
  align-items: center;
  padding: 0 20px;
  font-size: 18px;
  font-weight: 700;
  color: #ffffff;
  background: #1f2933;
}
:deep(.el-aside) {
  background: #1f2933;
  border-right: none;
}
:deep(.el-aside .el-menu) {
  background: transparent;
  border-right: none;
}
:deep(.el-aside .el-menu-item),
:deep(.el-aside .el-menu-item-group__title) {
  color: #cdd5e0;
}
:deep(.el-aside .el-menu-item:hover),
:deep(.el-aside .el-menu-item.is-active) {
  background: #2d3a4a;
  color: #ffffff;
}
:deep(.el-aside .el-menu-item-group__title) {
  color: #8a98a8;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding-top: 12px;
}
.header { display: flex; align-items: center; justify-content: space-between; background: var(--panel); border-bottom: 1px solid var(--border); }
.user-menu { display: flex; align-items: center; gap: 8px; cursor: pointer; }
.main { padding: 20px; }
</style>
