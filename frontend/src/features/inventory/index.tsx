import React from 'react';
import { Routes, Route } from 'react-router-dom';

const InboundListPage = React.lazy(() => import('./pages/InboundListPage'));
const InboundFormPage = React.lazy(() => import('./pages/InboundFormPage'));
const OutboundListPage = React.lazy(() => import('./pages/OutboundListPage'));
const OutboundFormPage = React.lazy(() => import('./pages/OutboundFormPage'));
const StockSummaryPage = React.lazy(() => import('./pages/StockSummaryPage'));

export default function InventoryModule() {
  return (
    <Routes>
      <Route index element={<StockSummaryPage />} />
      <Route path="stock" element={<StockSummaryPage />} />
      <Route path="inbound" element={<InboundListPage />} />
      <Route path="inbound/new" element={<InboundFormPage />} />
      <Route path="outbound" element={<OutboundListPage />} />
      <Route path="outbound/new" element={<OutboundFormPage />} />
    </Routes>
  );
}
