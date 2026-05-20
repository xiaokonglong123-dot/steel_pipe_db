import React from 'react';
import { Routes, Route } from 'react-router-dom';

const SalesOrderListPage = React.lazy(() => import('./pages/SalesOrderListPage'));
const SalesOrderFormPage = React.lazy(() => import('./pages/SalesOrderFormPage'));
const SalesOrderDetailPage = React.lazy(() => import('./pages/SalesOrderDetailPage'));
const CustomerListPage = React.lazy(() => import('./pages/CustomerListPage'));
const CustomerFormPage = React.lazy(() => import('./pages/CustomerFormPage'));

export default function SalesModule() {
  return (
    <Routes>
      <Route index element={<SalesOrderListPage />} />
      <Route path="new" element={<SalesOrderFormPage />} />
      <Route path=":id" element={<SalesOrderDetailPage />} />
      <Route path=":id/edit" element={<SalesOrderFormPage />} />
      <Route path="customers" element={<CustomerListPage />} />
      <Route path="customers/new" element={<CustomerFormPage />} />
      <Route path="customers/:id/edit" element={<CustomerFormPage />} />
    </Routes>
  );
}
