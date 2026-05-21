import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  inboundApi,
  outboundApi,
  inventoryApi,
  locationApi,
  checkApi,
  pipeSearchApi,
} from '../api/inventoryApi';
import type {
  InboundFilter,
  OutboundFilter,
  StockFilter,
  LocationFilter,
  CheckFilter,
  CreateInboundData,
  CreateOutboundData,
  CreateLocationData,
  UpdateLocationData,
  CreateCheckData,
  SubmitCheckItemData,
  PipeSearchResult,
} from '../api/inventoryApi';

// ━━━ Inbound ━━━

export function useInboundRecords(params?: InboundFilter) {
  return useQuery({
    queryKey: ['inbound-records', params],
    queryFn: () => inboundApi.list(params),
  });
}

export function useInboundRecord(id: number) {
  return useQuery({
    queryKey: ['inbound-record', id],
    queryFn: () => inboundApi.get(id),
    enabled: !!id,
  });
}

export function useCreateInbound() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateInboundData) => inboundApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['inbound-records'] });
    },
  });
}

export function useApproveInbound() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, reason }: { id: number; reason?: string }) =>
      inboundApi.approve(id, reason),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['inbound-records'] });
      qc.invalidateQueries({ queryKey: ['inbound-record'] });
    },
  });
}

export function useRejectInbound() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, reason }: { id: number; reason: string }) =>
      inboundApi.reject(id, reason),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['inbound-records'] });
      qc.invalidateQueries({ queryKey: ['inbound-record'] });
    },
  });
}

export function useDeleteInbound() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => inboundApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['inbound-records'] });
    },
  });
}

// ━━━ Outbound ━━━

export function useOutboundRecords(params?: OutboundFilter) {
  return useQuery({
    queryKey: ['outbound-records', params],
    queryFn: () => outboundApi.list(params),
  });
}

export function useOutboundRecord(id: number) {
  return useQuery({
    queryKey: ['outbound-record', id],
    queryFn: () => outboundApi.get(id),
    enabled: !!id,
  });
}

export function useCreateOutbound() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateOutboundData) => outboundApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['outbound-records'] });
    },
  });
}

export function useApproveOutbound() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, reason }: { id: number; reason?: string }) =>
      outboundApi.approve(id, reason),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['outbound-records'] });
      qc.invalidateQueries({ queryKey: ['outbound-record'] });
    },
  });
}

export function useRejectOutbound() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, reason }: { id: number; reason: string }) =>
      outboundApi.reject(id, reason),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['outbound-records'] });
      qc.invalidateQueries({ queryKey: ['outbound-record'] });
    },
  });
}

export function useDeleteOutbound() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => outboundApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['outbound-records'] });
    },
  });
}

// ━━━ Inventory / Stock ━━━

export function useStockQuery(params?: StockFilter) {
  return useQuery({
    queryKey: ['inventory-stock', params],
    queryFn: () => inventoryApi.queryStock(params),
  });
}

export function useInventoryLogs(params?: StockFilter) {
  return useQuery({
    queryKey: ['inventory-logs', params],
    queryFn: () => inventoryApi.queryLogs(params),
  });
}

export function useTracePipe(pipeType: string, pipeId: number) {
  return useQuery({
    queryKey: ['trace-pipe', pipeType, pipeId],
    queryFn: () => inventoryApi.tracePipe(pipeType, pipeId),
    enabled: !!pipeType && !!pipeId,
  });
}

export function useTraceHeat(heatNumber: string) {
  return useQuery({
    queryKey: ['trace-heat', heatNumber],
    queryFn: () => inventoryApi.traceHeat(heatNumber),
    enabled: !!heatNumber,
  });
}

export function useTraceOrder(orderType: string, orderId: number) {
  return useQuery({
    queryKey: ['trace-order', orderType, orderId],
    queryFn: () => inventoryApi.traceOrder(orderType, orderId),
    enabled: !!orderType && !!orderId,
  });
}

// ━━━ Locations ━━━

export function useLocations(params?: LocationFilter) {
  return useQuery({
    queryKey: ['locations', params],
    queryFn: () => locationApi.list(params),
  });
}

export function useLocation(id: number) {
  return useQuery({
    queryKey: ['location', id],
    queryFn: () => locationApi.get(id),
    enabled: !!id,
  });
}

export function useCreateLocation() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateLocationData) => locationApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['locations'] });
    },
  });
}

export function useUpdateLocation() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: number; data: UpdateLocationData }) =>
      locationApi.update(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['locations'] });
      qc.invalidateQueries({ queryKey: ['location'] });
    },
  });
}

export function useDeleteLocation() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => locationApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['locations'] });
    },
  });
}

// ━━━ Check ━━━

export function useInventoryChecks(params?: CheckFilter) {
  return useQuery({
    queryKey: ['inventory-checks', params],
    queryFn: () => checkApi.list(params),
  });
}

export function useInventoryCheck(id: number) {
  return useQuery({
    queryKey: ['inventory-check', id],
    queryFn: () => checkApi.get(id),
    enabled: !!id,
  });
}

export function useCreateCheck() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateCheckData) => checkApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['inventory-checks'] });
    },
  });
}

export function useSubmitCheckItem() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      checkId,
      itemId,
      data,
    }: {
      checkId: number;
      itemId: number;
      data: SubmitCheckItemData;
    }) => checkApi.submitItem(checkId, itemId, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['inventory-check'] });
    },
  });
}

// ━━━ Pipe Search ━━━

export function usePipeSearch(params?: { q?: string; pipe_type?: string; status?: string }) {
  return useQuery({
    queryKey: ['pipe-search', params],
    queryFn: () => pipeSearchApi.search(params),
  });
}

export type { PipeSearchResult };
