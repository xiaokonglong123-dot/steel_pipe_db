import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { message } from 'antd';
import { useTranslation } from 'react-i18next';
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
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: BatchLabelRequest) => labelApi.createBatchLabels(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: labelQueryKeys.pipe.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
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
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: ShippingLabelRequest) => labelApi.createShippingLabel(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: labelQueryKeys.shipping.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}
