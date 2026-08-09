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
const RoleManagementPage = lazy(() => import('@/features/auth/pages/RoleManagementPage'));
const DepartmentPage = lazy(() => import('@/features/auth/pages/DepartmentPage'));
const WorkflowListPage = lazy(() => import('@/features/workflow/pages/WorkflowListPage'));
const MyTasksPage = lazy(() => import('@/features/workflow/pages/MyTasksPage'));
const EmployeeListPage = lazy(() => import('@/features/hr/pages/EmployeeListPage'));
const SalaryPage = lazy(() => import('@/features/hr/pages/SalaryPage'));
const FinancePage = lazy(() => import('@/features/finance/pages/FinancePage'));
const ProcurementPage = lazy(() => import('@/features/procurement/pages/ProcurementPage'));
const SalesCrmPage = lazy(() => import('@/features/sales_crm/pages/SalesCrmPage'));
const InventoryAtpPage = lazy(() => import('@/features/inventory_atp/pages/InventoryAtpPage'));
const ManufacturingPage = lazy(() => import('@/features/manufacturing/pages/ManufacturingPage'));
const ProjectPage = lazy(() => import('@/features/project/pages/ProjectPage'));
const AssetsPage = lazy(() => import('@/features/assets/pages/AssetsPage'));
const NotificationsPage = lazy(() => import('@/features/notification/pages/NotificationsPage'));
const BiDashboardPage = lazy(() => import('@/features/bi/pages/DashboardPage'));
const PortalAdminPage = lazy(() => import('@/features/portal/pages/PortalAdminPage'));
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
      { index: true, element: <Navigate to="/inventory/inbound" replace /> },
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
      { path: 'data-io/import', element: route(<DataImportPage />), handle: { roles: ['admin'] } },
      { path: 'data-io/export', element: route(<DataExportPage />), handle: { roles: ['admin'] } },
      { path: 'data-io/logs', element: route(<OperationLogPage />), handle: { roles: ['admin'] } },
      // System
      { path: 'system/users', element: route(<UserManagementPage />), handle: { roles: ['admin'] } },
      { path: 'system/roles', element: route(<RoleManagementPage />), handle: { roles: ['admin'] } },
      { path: 'system/departments', element: route(<DepartmentPage />), handle: { roles: ['admin'] } },
      // Workflow
      { path: 'workflow/definitions', element: route(<WorkflowListPage />), handle: { roles: ['admin'] } },
      { path: 'workflow/my-tasks', element: route(<MyTasksPage />) },
      // HR
      { path: 'hr/employees', element: route(<EmployeeListPage />), handle: { roles: ['admin'] } },
      { path: 'hr/salaries', element: route(<SalaryPage />), handle: { roles: ['admin'] } },
      // Finance
      { path: 'finance', element: route(<FinancePage />), handle: { roles: ['admin'] } },
      // Procurement
      { path: 'procurement', element: route(<ProcurementPage />), handle: { roles: ['admin'] } },
      // Sales CRM
      { path: 'sales/crm', element: route(<SalesCrmPage />), handle: { roles: ['admin'] } },
      // Inventory ATP
      { path: 'inventory/atp', element: route(<InventoryAtpPage />), handle: { roles: ['admin'] } },
      // Manufacturing
      { path: 'manufacturing', element: route(<ManufacturingPage />), handle: { roles: ['admin'] } },
      // Projects
      { path: 'projects', element: route(<ProjectPage />), handle: { roles: ['admin'] } },
      // Assets
      { path: 'assets', element: route(<AssetsPage />), handle: { roles: ['admin'] } },
      // Notifications
      { path: 'notifications', element: route(<NotificationsPage />) },
      // BI Dashboard
      { path: 'bi', element: route(<BiDashboardPage />), handle: { roles: ['admin'] } },
      // Portal admin
      { path: 'portal', element: route(<PortalAdminPage />), handle: { roles: ['admin'] } },
      // Search & profile
      { path: 'search', element: route(<SearchPage />) },
      { path: 'profile/settings', element: route(<ProfileSettingsPage />) },
      { path: '*', element: <Navigate to="/" replace /> },
    ],
  },
]);
