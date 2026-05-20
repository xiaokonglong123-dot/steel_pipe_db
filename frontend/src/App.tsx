import React from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { useAuthStore } from './stores/authStore';
import MainLayout from './layouts/MainLayout';
import LoginPage from './features/auth/pages/LoginPage';

const LazyPipes = React.lazy(() => import('./features/pipes'));
const LazyInventory = React.lazy(() => import('./features/inventory'));
const LazyQuality = React.lazy(() => import('./features/quality'));
const LazyPurchases = React.lazy(() => import('./features/purchases'));
const LazySales = React.lazy(() => import('./features/sales'));
const LazyDataIo = React.lazy(() => import('./features/data-io'));
const LazyContracts = React.lazy(() => import('./features/contracts'));
const LazyReports = React.lazy(() => import('./features/reports'));
const LazyLabels = React.lazy(() => import('./features/labels'));
const LazyAuth = React.lazy(() => import('./features/auth'));

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  if (!isAuthenticated) return <Navigate to="/login" replace />;
  return <>{children}</>;
}

function ProtectedLayout({ children }: { children: React.ReactNode }) {
  return (
    <ProtectedRoute>
      <MainLayout>{children}</MainLayout>
    </ProtectedRoute>
  );
}

export default function App() {
  return (
    <React.Suspense fallback={<div style={{ padding: 48, textAlign: 'center' }}>Loading...</div>}>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/" element={<ProtectedLayout><Navigate to="/pipes" replace /></ProtectedLayout>} />
        <Route path="/pipes/*" element={<ProtectedLayout><LazyPipes /></ProtectedLayout>} />
        <Route path="/inventory/*" element={<ProtectedLayout><LazyInventory /></ProtectedLayout>} />
        <Route path="/quality/*" element={<ProtectedLayout><LazyQuality /></ProtectedLayout>} />
        <Route path="/purchases/*" element={<ProtectedLayout><LazyPurchases /></ProtectedLayout>} />
        <Route path="/sales/*" element={<ProtectedLayout><LazySales /></ProtectedLayout>} />
        <Route path="/data-io/*" element={<ProtectedLayout><LazyDataIo /></ProtectedLayout>} />
        <Route path="/contracts/*" element={<ProtectedLayout><LazyContracts /></ProtectedLayout>} />
        <Route path="/reports/*" element={<ProtectedLayout><LazyReports /></ProtectedLayout>} />
        <Route path="/labels/*" element={<ProtectedLayout><LazyLabels /></ProtectedLayout>} />
        <Route path="/system/*" element={<ProtectedLayout><LazyAuth /></ProtectedLayout>} />
      </Routes>
    </React.Suspense>
  );
}
