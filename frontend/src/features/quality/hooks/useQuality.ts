import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { message } from 'antd';
import { useTranslation } from 'react-i18next';
import { qualityApi } from '../api/qualityApi';
import { qualityQueryKeys } from '../queryKeys';
import type { CreateQualityCertData, CertFilterParams } from '../types';

export function useCerts(params?: CertFilterParams) {
  return useQuery({
    queryKey: qualityQueryKeys.certs.list(params),
    queryFn: () => qualityApi.getCerts(params),
  });
}

export function useCert(id: number) {
  return useQuery({
    queryKey: qualityQueryKeys.certs.detail(id),
    queryFn: () => qualityApi.getCert(id),
    enabled: !!id,
  });
}

export function useCreateCert() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: CreateQualityCertData) => qualityApi.createCert(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qualityQueryKeys.certs.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useUpdateCert(id: number) {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: Partial<CreateQualityCertData>) => qualityApi.updateCert(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qualityQueryKeys.certs.all });
      qc.invalidateQueries({ queryKey: qualityQueryKeys.certs.detail(id) });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useDeleteCert() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (id: number) => qualityApi.deleteCert(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qualityQueryKeys.certs.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useGrades() {
  return useQuery({
    queryKey: qualityQueryKeys.grades(),
    queryFn: () => qualityApi.getGrades(),
  });
}

export function useAttachments(certId: number) {
  return useQuery({
    queryKey: qualityQueryKeys.attachments.list(certId),
    queryFn: () => qualityApi.getAttachments(certId),
    enabled: !!certId,
  });
}

export function useCreateAttachment() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (data: { pipe_type: string; pipe_id: number; file_name: string; file_path: string; file_size?: number; content_type?: string }) => qualityApi.createAttachment(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qualityQueryKeys.attachments.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}

export function useDeleteAttachment() {
  const qc = useQueryClient();
  const { t } = useTranslation();
  return useMutation({
    mutationFn: (id: number) => qualityApi.deleteAttachment(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qualityQueryKeys.attachments.all });
    },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
}
