// Root component — mounts Ant Design theme + TanStack Query global config + router
import { ConfigProvider } from 'antd';
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
          <RouterProvider router={router} />
        </QueryClientProvider>
      </ConfigProvider>
    </ErrorBoundary>
  );
}
