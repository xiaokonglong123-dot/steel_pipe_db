import type {
  CheckFilter,
  InboundFilter,
  LocationFilter,
  OutboundFilter,
  StockFilter,
} from './api/inventoryApi';

export const inventoryQueryKeys = {
  inbound: {
    all: ['inbound-records'] as const,
    list: (params?: InboundFilter) => [...inventoryQueryKeys.inbound.all, params] as const,
    detail: (id: number) => ['inbound-record', id] as const,
    details: ['inbound-record'] as const,
  },
  outbound: {
    all: ['outbound-records'] as const,
    list: (params?: OutboundFilter) => [...inventoryQueryKeys.outbound.all, params] as const,
    detail: (id: number) => ['outbound-record', id] as const,
    details: ['outbound-record'] as const,
  },
  stock: {
    list: (params?: StockFilter) => ['inventory-stock', params] as const,
    logs: (params?: StockFilter) => ['inventory-logs', params] as const,
  },
  trace: {
    pipe: (pipeType: string, pipeId: number) => ['trace-pipe', pipeType, pipeId] as const,
    heat: (heatNumber: string) => ['trace-heat', heatNumber] as const,
    order: (orderType: string, orderId: number) => ['trace-order', orderType, orderId] as const,
  },
  locations: {
    all: ['locations'] as const,
    list: (params?: LocationFilter) => [...inventoryQueryKeys.locations.all, params] as const,
    detail: (id: number) => ['location', id] as const,
    details: ['location'] as const,
  },
  checks: {
    all: ['inventory-checks'] as const,
    list: (params?: CheckFilter) => [...inventoryQueryKeys.checks.all, params] as const,
    detail: (id: number) => ['inventory-check', id] as const,
    details: ['inventory-check'] as const,
  },
  pipeSearch: (params?: { q?: string; pipe_type?: string; status?: string }) =>
    ['pipe-search', params] as const,
};
