// 前端路由配置 — createBrowserRouter，公共路由 + 受保护的业务路由
// 所有业务页面挂载在 ProtectedRoute + MainLayout 下，自动带侧边栏和顶栏
import { createBrowserRouter, Navigate } from 'react-router-dom';
import MainLayout from '@/layouts/MainLayout';
import ProtectedRoute from './ProtectedRoute';
import LoginPage from '@/features/auth/pages/LoginPage';
import SeamlessPipeListPage from '@/features/pipes/pages/SeamlessPipeListPage';
import SeamlessPipeFormPage from '@/features/pipes/pages/SeamlessPipeFormPage';
import SeamlessPipeDetailPage from '@/features/pipes/pages/SeamlessPipeDetailPage';
import ScreenPipeListPage from '@/features/pipes/pages/ScreenPipeListPage';
import ScreenPipeFormPage from '@/features/pipes/pages/ScreenPipeFormPage';
import ScreenPipeDetailPage from '@/features/pipes/pages/ScreenPipeDetailPage';
import InboundListPage from '@/features/inventory/pages/InboundListPage';
import InboundFormPage from '@/features/inventory/pages/InboundFormPage';
import OutboundListPage from '@/features/inventory/pages/OutboundListPage';
import OutboundFormPage from '@/features/inventory/pages/OutboundFormPage';
import StockQueryPage from '@/features/inventory/pages/StockQueryPage';
import LocationListPage from '@/features/inventory/pages/LocationListPage';
import InventoryCheckListPage from '@/features/inventory/pages/InventoryCheckListPage';
import SupplierListPage from '@/features/suppliers/pages/SupplierListPage';
import SupplierFormPage from '@/features/suppliers/pages/SupplierFormPage';
import CustomerListPage from '@/features/customers/pages/CustomerListPage';
import CustomerFormPage from '@/features/customers/pages/CustomerFormPage';
import PurchaseOrderListPage from '@/features/purchases/pages/PurchaseOrderListPage';
import PurchaseOrderFormPage from '@/features/purchases/pages/PurchaseOrderFormPage';
import PurchaseOrderDetailPage from '@/features/purchases/pages/PurchaseOrderDetailPage';
import SalesOrderListPage from '@/features/sales/pages/SalesOrderListPage';
import SalesOrderFormPage from '@/features/sales/pages/SalesOrderFormPage';
import SalesOrderDetailPage from '@/features/sales/pages/SalesOrderDetailPage';
import CertListPage from '@/features/quality/pages/CertListPage';
import CertFormPage from '@/features/quality/pages/CertFormPage';
import CertDetailPage from '@/features/quality/pages/CertDetailPage';
import ContractListPage from '@/features/contracts/pages/ContractListPage';
import ContractFormPage from '@/features/contracts/pages/ContractFormPage';
import ContractDetailPage from '@/features/contracts/pages/ContractDetailPage';
import ReportListPage from '@/features/reports/pages/ReportListPage';
import DashboardPage from '@/features/reports/pages/DashboardPage';
import LabelPrintPage from '@/features/labels/pages/LabelPrintPage';
import ProfileSettingsPage from '@/features/profile/pages/ProfileSettingsPage';
import SearchPage from '@/features/search/pages/SearchPage';
import UserManagementPage from '@/features/auth/pages/UserManagementPage';

export const router = createBrowserRouter([
  {
    path: '/login',
    element: <LoginPage />,
  },
  {
    path: '/',
    element: (
      <ProtectedRoute>
        <MainLayout />
      </ProtectedRoute>
    ),
    children: [
      // 首页默认跳转到无缝钢管列表
      { index: true, element: <Navigate to="/pipes/seamless" replace /> },
      // --- 钢管管理 ---
      {
        path: 'pipes/seamless',
        element: <SeamlessPipeListPage />,
      },
      {
        path: 'pipes/seamless/new',
        element: <SeamlessPipeFormPage />,
      },
      {
        path: 'pipes/seamless/:id',
        element: <SeamlessPipeDetailPage />,
      },
      {
        path: 'pipes/seamless/:id/edit',
        element: <SeamlessPipeFormPage />,
      },
      {
        path: 'pipes/screen',
        element: <ScreenPipeListPage />,
      },
      {
        path: 'pipes/screen/new',
        element: <ScreenPipeFormPage />,
      },
      {
        path: 'pipes/screen/:id',
        element: <ScreenPipeDetailPage />,
      },
      {
        path: 'pipes/screen/:id/edit',
        element: <ScreenPipeFormPage />,
      },
      // --- 库存管理：入库/出库/库存查询/库位/盘点 ---
      { path: 'inventory/inbound', element: <InboundListPage /> },
      { path: 'inventory/inbound/new', element: <InboundFormPage key="new" /> },
      { path: 'inventory/inbound/:id/edit', element: <InboundFormPage key="edit" /> },
      { path: 'inventory/outbound', element: <OutboundListPage /> },
      { path: 'inventory/outbound/new', element: <OutboundFormPage key="new" /> },
      { path: 'inventory/outbound/:id/edit', element: <OutboundFormPage key="edit" /> },
      { path: 'inventory/stock', element: <StockQueryPage /> },
      { path: 'inventory/locations', element: <LocationListPage /> },
      { path: 'inventory/check', element: <InventoryCheckListPage /> },
      // --- 供应商与客户管理 ---
      { path: 'suppliers', element: <SupplierListPage /> },
      { path: 'suppliers/new', element: <SupplierFormPage key="new" /> },
      { path: 'suppliers/:id/edit', element: <SupplierFormPage key="edit" /> },
      { path: 'customers', element: <CustomerListPage /> },
      { path: 'customers/new', element: <CustomerFormPage key="new" /> },
      { path: 'customers/:id/edit', element: <CustomerFormPage key="edit" /> },
      // --- 采购订单 ---
      { path: 'purchases', element: <PurchaseOrderListPage /> },
      { path: 'purchases/new', element: <PurchaseOrderFormPage key="new" /> },
      { path: 'purchases/:id', element: <PurchaseOrderDetailPage /> },
      { path: 'purchases/:id/edit', element: <PurchaseOrderFormPage key="edit" /> },
      // --- 销售订单 ---
      { path: 'sales', element: <SalesOrderListPage /> },
      { path: 'sales/new', element: <SalesOrderFormPage key="new" /> },
      { path: 'sales/:id', element: <SalesOrderDetailPage /> },
      { path: 'sales/:id/edit', element: <SalesOrderFormPage key="edit" /> },
      // --- 质量证书 ---
      { path: 'quality/certs', element: <CertListPage /> },
      { path: 'quality/certs/new', element: <CertFormPage key="new" /> },
      { path: 'quality/certs/:id', element: <CertDetailPage /> },
      { path: 'quality/certs/:id/edit', element: <CertFormPage key="edit" /> },
      // --- 合同管理 ---
      { path: 'contracts', element: <ContractListPage /> },
      { path: 'contracts/new', element: <ContractFormPage key="new" /> },
      { path: 'contracts/:id', element: <ContractDetailPage /> },
      { path: 'contracts/:id/edit', element: <ContractFormPage key="edit" /> },
      // --- 报表与标签打印 ---
      { path: 'reports', element: <ReportListPage /> },
      { path: 'reports/dashboard', element: <DashboardPage /> },
      { path: 'labels', element: <LabelPrintPage /> },
      // --- 系统管理 ---
      { path: 'system/users', element: <UserManagementPage /> },
      // --- 全局搜索与个人设置 ---
      { path: 'search', element: <SearchPage /> },
      { path: 'profile/settings', element: <ProfileSettingsPage /> },
    ],
  },
]);
