// Root component — mounts Ant Design theme + TanStack Query global config + router
// RouterProvider consumes createBrowserRouter from routes/index.tsx
import { Suspense } from 'react';
import { ConfigProvider, Spin } from 'antd';
import { QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from 'react-router-dom';
import { queryClient } from '@/api/queryClient';
import { theme } from '@/styles/theme';
import { router } from '@/routes';
import ErrorBoundary from '@/shared/components/ErrorBoundary';
import { useRestoreSession } from '@/features/auth/hooks/useAuth';

function RestoreSession() {
  useRestoreSession();
  return null;
}

export default function App() {
  return (
    <ErrorBoundary>
      <ConfigProvider theme={theme}>
        <QueryClientProvider client={queryClient}>
          <RestoreSession />
          <Suspense fallback={<div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}><Spin size="large" /></div>}>
            <RouterProvider router={router} />
          </Suspense>
        </QueryClientProvider>
      </ConfigProvider>
    </ErrorBoundary>
  );
}
