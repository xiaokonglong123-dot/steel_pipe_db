import React from 'react';
import { Routes, Route } from 'react-router-dom';

const QualityListPage = React.lazy(() => import('./pages/QualityListPage'));
const QualityFormPage = React.lazy(() => import('./pages/QualityFormPage'));
const QualityDetailPage = React.lazy(() => import('./pages/QualityDetailPage'));

export default function QualityModule() {
  return (
    <Routes>
      <Route index element={<QualityListPage />} />
      <Route path="certs" element={<QualityListPage />} />
      <Route path="certs/new" element={<QualityFormPage />} />
      <Route path="certs/:id" element={<QualityDetailPage />} />
      <Route path="certs/:id/edit" element={<QualityFormPage />} />
    </Routes>
  );
}
