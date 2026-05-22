// 全局搜索 API — 各模块统一搜索接口
import { useQuery } from '@tanstack/react-query';
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import type {
  SearchPipeResult,
  SearchInboundResult,
  SearchOutboundResult,
  SearchPurchaseOrderResult,
  SearchSalesOrderResult,
} from '../types';

export function useSearchPipes(query: string) {
  return useQuery({
    queryKey: ['search', 'pipes', query],
    queryFn: () =>
      apiClient
        .get<ApiResponse<SearchPipeResult[]>>('/pipes/search', { params: { q: query } })
        .then((r) => r.data.data),
    enabled: query.length > 0,
  });
}

export function useSearchInbound(query: string) {
  return useQuery({
    queryKey: ['search', 'inbound', query],
    queryFn: () =>
      apiClient
        .get<ApiResponse<SearchInboundResult[]>>('/inventory/inbound/search', { params: { q: query } })
        .then((r) => r.data.data),
    enabled: query.length > 0,
  });
}

export function useSearchOutbound(query: string) {
  return useQuery({
    queryKey: ['search', 'outbound', query],
    queryFn: () =>
      apiClient
        .get<ApiResponse<SearchOutboundResult[]>>('/inventory/outbound/search', { params: { q: query } })
        .then((r) => r.data.data),
    enabled: query.length > 0,
  });
}

export function useSearchPurchaseOrders(query: string) {
  return useQuery({
    queryKey: ['search', 'purchases', query],
    queryFn: () =>
      apiClient
        .get<ApiResponse<SearchPurchaseOrderResult[]>>('/purchase-orders/search', { params: { q: query } })
        .then((r) => r.data.data),
    enabled: query.length > 0,
  });
}

export function useSearchSalesOrders(query: string) {
  return useQuery({
    queryKey: ['search', 'sales', query],
    queryFn: () =>
      apiClient
        .get<ApiResponse<SearchSalesOrderResult[]>>('/sales-orders/search', { params: { q: query } })
        .then((r) => r.data.data),
    enabled: query.length > 0,
  });
}
