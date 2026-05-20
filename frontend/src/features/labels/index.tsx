import React from 'react';
import { Routes, Route } from 'react-router-dom';

const LabelTemplateListPage = React.lazy(() => import('./pages/LabelTemplateListPage'));
const LabelTemplateFormPage = React.lazy(() => import('./pages/LabelTemplateFormPage'));
const LabelGeneratePage = React.lazy(() => import('./pages/LabelGeneratePage'));

export default function LabelsModule() {
  return (
    <Routes>
      <Route index element={<LabelTemplateListPage />} />
      <Route path="templates" element={<LabelTemplateListPage />} />
      <Route path="templates/new" element={<LabelTemplateFormPage />} />
      <Route path="templates/:id/edit" element={<LabelTemplateFormPage />} />
      <Route path="generate" element={<LabelGeneratePage />} />
    </Routes>
  );
}
