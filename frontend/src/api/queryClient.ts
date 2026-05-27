/**
 * TanStack Query 全局配置
 *
 * - staleTime: 2 分钟（避免频繁重复请求）
 * - gcTime: 5 分钟（缓存保留时长）
 * - retry: 1 次（失败后自动重试一次）
 * - refetchOnWindowFocus: false（不因窗口聚焦而重新请求）
 * - mutation onError: 全局打印突变错误
 */
import { QueryClient } from '@tanstack/react-query';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 2 * 60 * 1000,
      gcTime: 5 * 60 * 1000,
      retry: 1,
      refetchOnWindowFocus: false,
    },
    mutations: {
      onError: (error) => {
        console.error('[Mutation Error]', error);
      },
    },
  },
});
