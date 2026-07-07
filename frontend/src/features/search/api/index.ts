import { useQuery } from '@tanstack/react-query';
import apiClient from '@/api/client';
import { validateResponse } from '@/lib/validateResponse';
import {
  searchPipeResultSchema,
  searchInboundResultSchema,
  searchOutboundResultSchema,
  searchPurchaseOrderResultSchema,
  searchSalesOrderResultSchema,
} from '@/zod-schemas/search';
import { searchQueryKeys } from '../queryKeys';
import { z } from 'zod';
import type { ApiResponse } from '@/types';

export function useSearchPipes(query: string, enabled = true) {
  return useQuery({
    queryKey: searchQueryKeys.pipes(query),
    queryFn: () =>
      apiClient
        .get<ApiResponse<unknown[]>>('/pipes/search', { q: query })
        .then((r) => validateResponse(z.array(searchPipeResultSchema), r.data)),
    enabled: query.length > 0 && enabled,
    staleTime: 0,
  });
}

export function useSearchInbound(query: string, enabled = true) {
  return useQuery({
    queryKey: searchQueryKeys.inbound(query),
    queryFn: () =>
      apiClient
        .get<ApiResponse<unknown[]>>('/inventory/inbound/search', { q: query })
        .then((r) => validateResponse(z.array(searchInboundResultSchema), r.data)),
    enabled: query.length > 0 && enabled,
    staleTime: 0,
  });
}

export function useSearchOutbound(query: string, enabled = true) {
  return useQuery({
    queryKey: searchQueryKeys.outbound(query),
    queryFn: () =>
      apiClient
        .get<ApiResponse<unknown[]>>('/inventory/outbound/search', { q: query })
        .then((r) => validateResponse(z.array(searchOutboundResultSchema), r.data)),
    enabled: query.length > 0 && enabled,
    staleTime: 0,
  });
}

export function useSearchPurchaseOrders(query: string, enabled = true) {
  return useQuery({
    queryKey: searchQueryKeys.purchases(query),
    queryFn: () =>
      apiClient
        .get<ApiResponse<unknown[]>>('/purchase-orders/search', { q: query })
        .then((r) => validateResponse(z.array(searchPurchaseOrderResultSchema), r.data)),
    enabled: query.length > 0 && enabled,
    staleTime: 0,
  });
}

export function useSearchSalesOrders(query: string, enabled = true) {
  return useQuery({
    queryKey: searchQueryKeys.sales(query),
    queryFn: () =>
      apiClient
        .get<ApiResponse<unknown[]>>('/sales-orders/search', { q: query })
        .then((r) => validateResponse(z.array(searchSalesOrderResultSchema), r.data)),
    enabled: query.length > 0 && enabled,
    staleTime: 0,
  });
}
