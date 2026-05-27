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

export function useSearchPipes(query: string) {
  return useQuery({
    queryKey: searchQueryKeys.pipes(query),
    queryFn: () =>
      apiClient
        .get('/pipes/search', { params: { q: query } })
        .then((r) => validateResponse(z.array(searchPipeResultSchema), r.data.data)),
    enabled: query.length > 0,
  });
}

export function useSearchInbound(query: string) {
  return useQuery({
    queryKey: searchQueryKeys.inbound(query),
    queryFn: () =>
      apiClient
        .get('/inventory/inbound/search', { params: { q: query } })
        .then((r) => validateResponse(z.array(searchInboundResultSchema), r.data.data)),
    enabled: query.length > 0,
  });
}

export function useSearchOutbound(query: string) {
  return useQuery({
    queryKey: searchQueryKeys.outbound(query),
    queryFn: () =>
      apiClient
        .get('/inventory/outbound/search', { params: { q: query } })
        .then((r) => validateResponse(z.array(searchOutboundResultSchema), r.data.data)),
    enabled: query.length > 0,
  });
}

export function useSearchPurchaseOrders(query: string) {
  return useQuery({
    queryKey: searchQueryKeys.purchases(query),
    queryFn: () =>
      apiClient
        .get('/purchase-orders/search', { params: { q: query } })
        .then((r) => validateResponse(z.array(searchPurchaseOrderResultSchema), r.data.data)),
    enabled: query.length > 0,
  });
}

export function useSearchSalesOrders(query: string) {
  return useQuery({
    queryKey: searchQueryKeys.sales(query),
    queryFn: () =>
      apiClient
        .get('/sales-orders/search', { params: { q: query } })
        .then((r) => validateResponse(z.array(searchSalesOrderResultSchema), r.data.data)),
    enabled: query.length > 0,
  });
}
