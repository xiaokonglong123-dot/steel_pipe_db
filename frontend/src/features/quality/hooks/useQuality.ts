import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
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
  return useMutation({
    mutationFn: (data: CreateQualityCertData) => qualityApi.createCert(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qualityQueryKeys.certs.all });
    },
  });
}

export function useUpdateCert(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: Partial<CreateQualityCertData>) => qualityApi.updateCert(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qualityQueryKeys.certs.all });
      qc.invalidateQueries({ queryKey: qualityQueryKeys.certs.detail(id) });
    },
  });
}

export function useDeleteCert() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => qualityApi.deleteCert(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qualityQueryKeys.certs.all });
    },
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
  return useMutation({
    mutationFn: (data: FormData) => qualityApi.createAttachment(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qualityQueryKeys.attachments.all });
    },
  });
}

export function useDeleteAttachment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => qualityApi.deleteAttachment(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qualityQueryKeys.attachments.all });
    },
  });
}
