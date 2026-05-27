// Root component — mounts Ant Design theme + TanStack Query global config + router
// RouterProvider consumes createBrowserRouter from routes/index.tsx
import { Suspense } from 'react';
import { ConfigProvider, Spin } from 'antd';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from 'react-router-dom';
import { theme } from '@/styles/theme';
import { router } from '@/routes';
import ErrorBoundary from '@/shared/components/ErrorBoundary';

// 2min staleTime works for most biz data; pages needing real-time should set refetchInterval individually
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 2 * 60 * 1000,
      gcTime: 5 * 60 * 1000,
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

export default function App() {
  return (
    <ErrorBoundary>
      <ConfigProvider theme={theme}>
        <QueryClientProvider client={queryClient}>
          <Suspense fallback={<div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}><Spin size="large" /></div>}>
            <RouterProvider router={router} />
          </Suspense>
        </QueryClientProvider>
      </ConfigProvider>
    </ErrorBoundary>
  );
}
