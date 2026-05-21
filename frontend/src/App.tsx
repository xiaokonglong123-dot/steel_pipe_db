// 根组件 — 挂载 Ant Design 主题 + TanStack Query 全局配置
// Outlet 由 react-router 注入子路由内容（MainLayout 或 LoginPage）
import { ConfigProvider } from 'antd';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Outlet } from 'react-router-dom';
import { theme } from '@/styles/theme';

// TODO: staleTime 2 分钟适合大部分业务数据，实时性要求高的页面应单独设置 refetchInterval
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
    <ConfigProvider theme={theme}>
      <QueryClientProvider client={queryClient}>
        <Outlet />
      </QueryClientProvider>
    </ConfigProvider>
  );
}
