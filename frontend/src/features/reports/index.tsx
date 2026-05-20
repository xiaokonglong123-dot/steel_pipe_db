import React from 'react';
import { Routes, Route } from 'react-router-dom';

const ReportDashboardPage = React.lazy(() => import('./pages/ReportDashboardPage'));
const StockReportPage = React.lazy(() => import('./pages/StockReportPage'));
const InboundReportPage = React.lazy(() => import('./pages/InboundReportPage'));
const OutboundReportPage = React.lazy(() => import('./pages/OutboundReportPage'));
const MonthlyFlowPage = React.lazy(() => import('./pages/MonthlyFlowPage'));
const PurchaseReportPage = React.lazy(() => import('./pages/PurchaseReportPage'));
const SalesReportPage = React.lazy(() => import('./pages/SalesReportPage'));

export default function ReportsModule() {
  return (
    <Routes>
      <Route index element={<ReportDashboardPage />} />
      <Route path="stock" element={<StockReportPage />} />
      <Route path="inbound" element={<InboundReportPage />} />
      <Route path="outbound" element={<OutboundReportPage />} />
      <Route path="monthly-flow" element={<MonthlyFlowPage />} />
      <Route path="purchase" element={<PurchaseReportPage />} />
      <Route path="sales" element={<SalesReportPage />} />
    </Routes>
  );
}
