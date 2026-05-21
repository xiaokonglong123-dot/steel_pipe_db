import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { labelApi } from '../api/labelApi';
import type { BatchLabelRequest, ShippingLabelRequest } from '../types';

export function usePipeLabel(pipeType: string, pipeId: number) {
  return useQuery({
    queryKey: ['pipe-label', pipeType, pipeId],
    queryFn: () => labelApi.getPipeLabel(pipeType, pipeId),
    enabled: !!pipeType && !!pipeId,
  });
}

export function useCreateBatchLabels() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: BatchLabelRequest) => labelApi.createBatchLabels(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['pipe-label'] });
    },
  });
}

export function useQualityLabel(certId: number) {
  return useQuery({
    queryKey: ['quality-label', certId],
    queryFn: () => labelApi.getQualityLabel(certId),
    enabled: !!certId,
  });
}

export function useCreateShippingLabel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: ShippingLabelRequest) => labelApi.createShippingLabel(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['shipping-label'] });
    },
  });
}
