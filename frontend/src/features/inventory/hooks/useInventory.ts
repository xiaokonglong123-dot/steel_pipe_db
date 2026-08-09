import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { message } from 'antd';
import { useTranslation } from 'react-i18next';
import {
  inboundApi,
  outboundApi,
  inventoryApi,
  locationApi,
  checkApi,
} from '../api/inventoryApi';
import { inventoryQueryKeys } from '../queryKeys';
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
} from '../api/inventoryApi';

// ━━━ Inbound ━━━

export function useInboundRecords(params?: InboundFilter) {
  return useQuery({
    queryKey: inventoryQueryKeys.inbound.list(params),
    queryFn: () => inboundApi.list(params),
  });
}

export function useInboundRecord(id: number) {
  return useQuery({
    queryKey: inventoryQueryKeys.inbound.detail(id),
    queryFn: () => inboundApi.get(id),
    enabled: !!id,
  });
}

export function useCreateInbound() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: CreateInboundData) => inboundApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.inbound.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useUpdateInbound(id: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: CreateInboundData) => inboundApi.update(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.inbound.all });
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.inbound.details });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useApproveInbound() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: ({ id, reason }: { id: number; reason?: string }) =>
      inboundApi.approve(id, reason),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.inbound.all });
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.inbound.details });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useRejectInbound() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: ({ id, reason }: { id: number; reason: string }) =>
      inboundApi.reject(id, reason),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.inbound.all });
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.inbound.details });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useDeleteInbound() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (id: number) => inboundApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.inbound.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

// ━━━ Outbound ━━━

export function useOutboundRecords(params?: OutboundFilter) {
  return useQuery({
    queryKey: inventoryQueryKeys.outbound.list(params),
    queryFn: () => outboundApi.list(params),
  });
}

export function useOutboundRecord(id: number) {
  return useQuery({
    queryKey: inventoryQueryKeys.outbound.detail(id),
    queryFn: () => outboundApi.get(id),
    enabled: !!id,
  });
}

export function useCreateOutbound() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: CreateOutboundData) => outboundApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.outbound.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useUpdateOutbound(id: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: CreateOutboundData) => outboundApi.update(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.outbound.all });
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.outbound.details });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useApproveOutbound() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: ({ id, reason }: { id: number; reason?: string }) =>
      outboundApi.approve(id, reason),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.outbound.all });
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.outbound.details });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useRejectOutbound() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: ({ id, reason }: { id: number; reason: string }) =>
      outboundApi.reject(id, reason),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.outbound.all });
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.outbound.details });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useDeleteOutbound() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (id: number) => outboundApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.outbound.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

// ━━━ Inventory / Stock ━━━

export function useStockQuery(params?: StockFilter) {
  return useQuery({
    queryKey: inventoryQueryKeys.stock.list(params),
    queryFn: () => inventoryApi.queryStock(params),
  });
}

export function useInventoryLogs(params?: StockFilter) {
  return useQuery({
    queryKey: inventoryQueryKeys.stock.logs(params),
    queryFn: () => inventoryApi.queryLogs(params),
  });
}

export function useTracePipe(pipeType: string, pipeId: number) {
  return useQuery({
    queryKey: inventoryQueryKeys.trace.pipe(pipeType, pipeId),
    queryFn: () => inventoryApi.tracePipe(pipeType, pipeId),
    enabled: !!pipeType && !!pipeId,
  });
}

export function useTraceHeat(heatNumber: string) {
  return useQuery({
    queryKey: inventoryQueryKeys.trace.heat(heatNumber),
    queryFn: () => inventoryApi.traceHeat(heatNumber),
    enabled: !!heatNumber,
  });
}

export function useTraceOrder(orderType: string, orderId: number) {
  return useQuery({
    queryKey: inventoryQueryKeys.trace.order(orderType, orderId),
    queryFn: () => inventoryApi.traceOrder(orderType, orderId),
    enabled: !!orderType && !!orderId,
  });
}

// ━━━ Locations ━━━

export function useLocations(params?: LocationFilter) {
  return useQuery({
    queryKey: inventoryQueryKeys.locations.list(params),
    queryFn: () => locationApi.list(params),
    staleTime: 10 * 60 * 1000,
  });
}

export function useLocation(id: number) {
  return useQuery({
    queryKey: inventoryQueryKeys.locations.detail(id),
    queryFn: () => locationApi.get(id),
    enabled: !!id,
  });
}

export function useCreateLocation() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: CreateLocationData) => locationApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.locations.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useUpdateLocation() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: ({ id, data }: { id: number; data: UpdateLocationData }) =>
      locationApi.update(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.locations.all });
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.locations.details });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useDeleteLocation() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (id: number) => locationApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.locations.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

// ━━━ Check ━━━

export function useInventoryChecks(params?: CheckFilter) {
  return useQuery({
    queryKey: inventoryQueryKeys.checks.list(params),
    queryFn: () => checkApi.list(params),
  });
}

export function useInventoryCheck(id: number) {
  return useQuery({
    queryKey: inventoryQueryKeys.checks.detail(id),
    queryFn: () => checkApi.get(id),
    enabled: !!id,
  });
}

export function useCreateCheck() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: CreateCheckData) => checkApi.create(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.checks.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useSubmitCheckItem() {
  const qc = useQueryClient();
  const { t } = useTranslation();
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
      qc.invalidateQueries({ queryKey: inventoryQueryKeys.checks.details });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}
