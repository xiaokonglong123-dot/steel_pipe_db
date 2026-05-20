import React from 'react';
import { Routes, Route } from 'react-router-dom';

const PipeListPage = React.lazy(() => import('./pages/PipeListPage'));
const PipeFormPage = React.lazy(() => import('./pages/PipeFormPage'));

export default function PipesModule() {
  return (
    <Routes>
      <Route index element={<PipeListPage />} />
      <Route path="new" element={<PipeFormPage />} />
      <Route path=":id/edit" element={<PipeFormPage />} />
    </Routes>
  );
}
