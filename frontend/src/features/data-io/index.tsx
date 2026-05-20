import React from 'react';
import { Routes, Route } from 'react-router-dom';

const ImportPage = React.lazy(() => import('./pages/ImportPage'));
const ExportPage = React.lazy(() => import('./pages/ExportPage'));

export default function DataIoModule() {
  return (
    <Routes>
      <Route index element={<ImportPage />} />
      <Route path="import" element={<ImportPage />} />
      <Route path="export" element={<ExportPage />} />
    </Routes>
  );
}
