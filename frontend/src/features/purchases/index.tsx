import React from 'react';
import { Routes, Route } from 'react-router-dom';

const PurchaseOrderListPage = React.lazy(() => import('./pages/PurchaseOrderListPage'));
const PurchaseOrderFormPage = React.lazy(() => import('./pages/PurchaseOrderFormPage'));
const PurchaseOrderDetailPage = React.lazy(() => import('./pages/PurchaseOrderDetailPage'));
const SupplierListPage = React.lazy(() => import('./pages/SupplierListPage'));
const SupplierFormPage = React.lazy(() => import('./pages/SupplierFormPage'));

export default function PurchasesModule() {
  return (
    <Routes>
      <Route index element={<PurchaseOrderListPage />} />
      <Route path="new" element={<PurchaseOrderFormPage />} />
      <Route path=":id" element={<PurchaseOrderDetailPage />} />
      <Route path=":id/edit" element={<PurchaseOrderFormPage />} />
      <Route path="suppliers" element={<SupplierListPage />} />
      <Route path="suppliers/new" element={<SupplierFormPage />} />
      <Route path="suppliers/:id/edit" element={<SupplierFormPage />} />
    </Routes>
  );
}
