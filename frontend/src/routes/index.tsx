/**
 * Route configuration with per-route Suspense + ErrorBoundary.
 *
 * Each page is lazy-loaded via React.lazy. The RouteBoundary wrapper
 * provides isolated error handling and loading states per route.
 */
import { lazy } from 'react';
import { createBrowserRouter, Navigate } from 'react-router-dom';
import MainLayout from '@/layouts/MainLayout';
import ProtectedRoute from './ProtectedRoute';
import { RouteBoundary } from '@/shared/components/RouteBoundary';

function route(element: React.ReactNode) {
  return <RouteBoundary>{element}</RouteBoundary>;
}

const LoginPage = lazy(() => import('@/features/auth/pages/LoginPage'));
const SeamlessPipeListPage = lazy(() => import('@/features/pipes/pages/SeamlessPipeListPage'));
const SeamlessPipeFormPage = lazy(() => import('@/features/pipes/pages/SeamlessPipeFormPage'));
const SeamlessPipeDetailPage = lazy(() => import('@/features/pipes/pages/SeamlessPipeDetailPage'));
const ScreenPipeListPage = lazy(() => import('@/features/pipes/pages/ScreenPipeListPage'));
const ScreenPipeFormPage = lazy(() => import('@/features/pipes/pages/ScreenPipeFormPage'));
const ScreenPipeDetailPage = lazy(() => import('@/features/pipes/pages/ScreenPipeDetailPage'));
const InboundListPage = lazy(() => import('@/features/inventory/pages/InboundListPage'));
const InboundFormPage = lazy(() => import('@/features/inventory/pages/InboundFormPage'));
const OutboundListPage = lazy(() => import('@/features/inventory/pages/OutboundListPage'));
const OutboundFormPage = lazy(() => import('@/features/inventory/pages/OutboundFormPage'));
const StockQueryPage = lazy(() => import('@/features/inventory/pages/StockQueryPage'));
const LocationListPage = lazy(() => import('@/features/inventory/pages/LocationListPage'));
const InventoryCheckListPage = lazy(() => import('@/features/inventory/pages/InventoryCheckListPage'));
const SupplierListPage = lazy(() => import('@/features/suppliers/pages/SupplierListPage'));
const SupplierFormPage = lazy(() => import('@/features/suppliers/pages/SupplierFormPage'));
const CustomerListPage = lazy(() => import('@/features/customers/pages/CustomerListPage'));
const CustomerFormPage = lazy(() => import('@/features/customers/pages/CustomerFormPage'));
const PurchaseOrderListPage = lazy(() => import('@/features/purchases/pages/PurchaseOrderListPage'));
const PurchaseOrderFormPage = lazy(() => import('@/features/purchases/pages/PurchaseOrderFormPage'));
const PurchaseOrderDetailPage = lazy(() => import('@/features/purchases/pages/PurchaseOrderDetailPage'));
const SalesOrderListPage = lazy(() => import('@/features/sales/pages/SalesOrderListPage'));
const SalesOrderFormPage = lazy(() => import('@/features/sales/pages/SalesOrderFormPage'));
const SalesOrderDetailPage = lazy(() => import('@/features/sales/pages/SalesOrderDetailPage'));
const CertListPage = lazy(() => import('@/features/quality/pages/CertListPage'));
const CertFormPage = lazy(() => import('@/features/quality/pages/CertFormPage'));
const CertDetailPage = lazy(() => import('@/features/quality/pages/CertDetailPage'));
const ContractListPage = lazy(() => import('@/features/contracts/pages/ContractListPage'));
const ContractFormPage = lazy(() => import('@/features/contracts/pages/ContractFormPage'));
const ContractDetailPage = lazy(() => import('@/features/contracts/pages/ContractDetailPage'));
const ReportListPage = lazy(() => import('@/features/reports/pages/ReportListPage'));
const DashboardPage = lazy(() => import('@/features/reports/pages/DashboardPage'));
const LabelPrintPage = lazy(() => import('@/features/labels/pages/LabelPrintPage'));
const ProfileSettingsPage = lazy(() => import('@/features/profile/pages/ProfileSettingsPage'));
const SearchPage = lazy(() => import('@/features/search/pages/SearchPage'));
const UserManagementPage = lazy(() => import('@/features/auth/pages/UserManagementPage'));
const DataImportPage = lazy(() => import('@/features/data-io/pages/DataImportPage'));
const DataExportPage = lazy(() => import('@/features/data-io/pages/DataExportPage'));
const OperationLogPage = lazy(() => import('@/features/data-io/pages/OperationLogPage'));
const InventoryReportPage = lazy(() => import('@/features/reports/pages/InventoryReportPage'));
const OrderReportPage = lazy(() => import('@/features/reports/pages/OrderReportPage'));
const QualityReportPage = lazy(() => import('@/features/reports/pages/QualityReportPage'));

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
      { index: true, element: <Navigate to="/pipes/seamless" replace /> },
      // Pipe management
      { path: 'pipes/seamless', element: route(<SeamlessPipeListPage />) },
      { path: 'pipes/seamless/new', element: route(<SeamlessPipeFormPage key="new" />) },
      { path: 'pipes/seamless/:id', element: route(<SeamlessPipeDetailPage />) },
      { path: 'pipes/seamless/:id/edit', element: route(<SeamlessPipeFormPage key="edit" />) },
      { path: 'pipes/screen', element: route(<ScreenPipeListPage />) },
      { path: 'pipes/screen/new', element: route(<ScreenPipeFormPage key="new" />) },
      { path: 'pipes/screen/:id', element: route(<ScreenPipeDetailPage />) },
      { path: 'pipes/screen/:id/edit', element: route(<ScreenPipeFormPage key="edit" />) },
      // Inventory
      { path: 'inventory/inbound', element: route(<InboundListPage />) },
      { path: 'inventory/inbound/new', element: route(<InboundFormPage key="new" />) },
      { path: 'inventory/inbound/:id/edit', element: route(<InboundFormPage key="edit" />) },
      { path: 'inventory/outbound', element: route(<OutboundListPage />) },
      { path: 'inventory/outbound/new', element: route(<OutboundFormPage key="new" />) },
      { path: 'inventory/outbound/:id/edit', element: route(<OutboundFormPage key="edit" />) },
      { path: 'inventory/stock', element: route(<StockQueryPage />) },
      { path: 'inventory/locations', element: route(<LocationListPage />) },
      { path: 'inventory/check', element: route(<InventoryCheckListPage />) },
      // Supplier & customer
      { path: 'suppliers', element: route(<SupplierListPage />) },
      { path: 'suppliers/new', element: route(<SupplierFormPage key="new" />) },
      { path: 'suppliers/:id/edit', element: route(<SupplierFormPage key="edit" />) },
      { path: 'customers', element: route(<CustomerListPage />) },
      { path: 'customers/new', element: route(<CustomerFormPage key="new" />) },
      { path: 'customers/:id/edit', element: route(<CustomerFormPage key="edit" />) },
      // Purchase orders
      { path: 'purchases', element: route(<PurchaseOrderListPage />) },
      { path: 'purchases/new', element: route(<PurchaseOrderFormPage key="new" />) },
      { path: 'purchases/:id', element: route(<PurchaseOrderDetailPage />) },
      { path: 'purchases/:id/edit', element: route(<PurchaseOrderFormPage key="edit" />) },
      // Sales orders
      { path: 'sales', element: route(<SalesOrderListPage />) },
      { path: 'sales/new', element: route(<SalesOrderFormPage key="new" />) },
      { path: 'sales/:id', element: route(<SalesOrderDetailPage />) },
      { path: 'sales/:id/edit', element: route(<SalesOrderFormPage key="edit" />) },
      // Quality certs
      { path: 'quality/certs', element: route(<CertListPage />) },
      { path: 'quality/certs/new', element: route(<CertFormPage key="new" />) },
      { path: 'quality/certs/:id', element: route(<CertDetailPage />) },
      { path: 'quality/certs/:id/edit', element: route(<CertFormPage key="edit" />) },
      // Contracts
      { path: 'contracts', element: route(<ContractListPage />) },
      { path: 'contracts/new', element: route(<ContractFormPage key="new" />) },
      { path: 'contracts/:id', element: route(<ContractDetailPage />) },
      { path: 'contracts/:id/edit', element: route(<ContractFormPage key="edit" />) },
      // Reports & labels
      { path: 'reports', element: route(<ReportListPage />) },
      { path: 'reports/dashboard', element: route(<DashboardPage />) },
      { path: 'reports/inventory', element: route(<InventoryReportPage />) },
      { path: 'reports/orders', element: route(<OrderReportPage />) },
      { path: 'reports/quality', element: route(<QualityReportPage />) },
      { path: 'labels', element: route(<LabelPrintPage />) },
      // Data IO
      { path: 'data-io/import', element: route(<DataImportPage />) },
      { path: 'data-io/export', element: route(<DataExportPage />) },
      { path: 'data-io/logs', element: route(<OperationLogPage />) },
      // System
      { path: 'system/users', element: route(<UserManagementPage />) },
      // Search & profile
      { path: 'search', element: route(<SearchPage />) },
      { path: 'profile/settings', element: route(<ProfileSettingsPage />) },
      { path: '*', element: <Navigate to="/" replace /> },
    ],
  },
]);
