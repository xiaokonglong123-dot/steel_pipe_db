import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { labelApi } from '../api/labelApi';
import { labelQueryKeys } from '../queryKeys';
import type { BatchLabelRequest, ShippingLabelRequest } from '../types';

export function usePipeLabel(pipeType: string, pipeId: number) {
  return useQuery({
    queryKey: labelQueryKeys.pipe.detail(pipeType, pipeId),
    queryFn: () => labelApi.getPipeLabel(pipeType, pipeId),
    enabled: !!pipeType && !!pipeId,
  });
}

export function useCreateBatchLabels() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: BatchLabelRequest) => labelApi.createBatchLabels(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: labelQueryKeys.pipe.all });
    },
  });
}

export function useQualityLabel(certId: number) {
  return useQuery({
    queryKey: labelQueryKeys.quality.detail(certId),
    queryFn: () => labelApi.getQualityLabel(certId),
    enabled: !!certId,
  });
}

export function useCreateShippingLabel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: ShippingLabelRequest) => labelApi.createShippingLabel(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: labelQueryKeys.shipping.all });
    },
  });
}
