import React from 'react';
import { Routes, Route } from 'react-router-dom';

const ContractListPage = React.lazy(() => import('./pages/ContractListPage'));
const ContractFormPage = React.lazy(() => import('./pages/ContractFormPage'));
const ContractDetailPage = React.lazy(() => import('./pages/ContractDetailPage'));

export default function ContractsModule() {
  return (
    <Routes>
      <Route index element={<ContractListPage />} />
      <Route path="new" element={<ContractFormPage />} />
      <Route path=":id" element={<ContractDetailPage />} />
      <Route path=":id/edit" element={<ContractFormPage />} />
    </Routes>
  );
}
