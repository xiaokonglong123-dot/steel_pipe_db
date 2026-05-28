/**
 * @module Frontend route configuration
 *
 * Defines the route structure using createBrowserRouter:
 * - Public routes: /login (no auth needed)
 * - Protected routes: all biz pages under ProtectedRoute + MainLayout, auto sidebar + header
 *   Default home redirects to /pipes/seamless
 *
 * All page components are lazy-loaded (React.lazy) so Vite code-splits them into
 * separate chunks — roughly one chunk per page. This keeps the initial vendor-antd
 * bundle from containing every antd component in the app.
 */
import { lazy } from 'react';
import { createBrowserRouter, Navigate } from 'react-router-dom';
import MainLayout from '@/layouts/MainLayout';
import ProtectedRoute from './ProtectedRoute';

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
      // Home defaults to seamless pipe list
      { index: true, element: <Navigate to="/pipes/seamless" replace /> },
      // Pipe management
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
      // Inventory: inbound/outbound/stock query/locations/stocktake
      { path: 'inventory/inbound', element: <InboundListPage /> },
      { path: 'inventory/inbound/new', element: <InboundFormPage key="new" /> },
      { path: 'inventory/inbound/:id/edit', element: <InboundFormPage key="edit" /> },
      { path: 'inventory/outbound', element: <OutboundListPage /> },
      { path: 'inventory/outbound/new', element: <OutboundFormPage key="new" /> },
      { path: 'inventory/outbound/:id/edit', element: <OutboundFormPage key="edit" /> },
      { path: 'inventory/stock', element: <StockQueryPage /> },
      { path: 'inventory/locations', element: <LocationListPage /> },
      { path: 'inventory/check', element: <InventoryCheckListPage /> },
      // Supplier & customer management
      { path: 'suppliers', element: <SupplierListPage /> },
      { path: 'suppliers/new', element: <SupplierFormPage key="new" /> },
      { path: 'suppliers/:id/edit', element: <SupplierFormPage key="edit" /> },
      { path: 'customers', element: <CustomerListPage /> },
      { path: 'customers/new', element: <CustomerFormPage key="new" /> },
      { path: 'customers/:id/edit', element: <CustomerFormPage key="edit" /> },
      // Purchase orders
      { path: 'purchases', element: <PurchaseOrderListPage /> },
      { path: 'purchases/new', element: <PurchaseOrderFormPage key="new" /> },
      { path: 'purchases/:id', element: <PurchaseOrderDetailPage /> },
      { path: 'purchases/:id/edit', element: <PurchaseOrderFormPage key="edit" /> },
      // Sales orders
      { path: 'sales', element: <SalesOrderListPage /> },
      { path: 'sales/new', element: <SalesOrderFormPage key="new" /> },
      { path: 'sales/:id', element: <SalesOrderDetailPage /> },
      { path: 'sales/:id/edit', element: <SalesOrderFormPage key="edit" /> },
      // Quality certs
      { path: 'quality/certs', element: <CertListPage /> },
      { path: 'quality/certs/new', element: <CertFormPage key="new" /> },
      { path: 'quality/certs/:id', element: <CertDetailPage /> },
      { path: 'quality/certs/:id/edit', element: <CertFormPage key="edit" /> },
      // Contracts
      { path: 'contracts', element: <ContractListPage /> },
      { path: 'contracts/new', element: <ContractFormPage key="new" /> },
      { path: 'contracts/:id', element: <ContractDetailPage /> },
      { path: 'contracts/:id/edit', element: <ContractFormPage key="edit" /> },
      // Reports & label printing
      { path: 'reports', element: <ReportListPage /> },
      { path: 'reports/dashboard', element: <DashboardPage /> },
      { path: 'reports/inventory', element: <InventoryReportPage /> },
      { path: 'reports/orders', element: <OrderReportPage /> },
      { path: 'reports/quality', element: <QualityReportPage /> },
      { path: 'labels', element: <LabelPrintPage /> },
      // Data IO: import/export/logs
      { path: 'data-io/import', element: <DataImportPage /> },
      { path: 'data-io/export', element: <DataExportPage /> },
      { path: 'data-io/logs', element: <OperationLogPage /> },
      // System management
      { path: 'system/users', element: <UserManagementPage /> },
      // Global search & profile settings
      { path: 'search', element: <SearchPage /> },
      { path: 'profile/settings', element: <ProfileSettingsPage /> },
    ],
  },
]);
