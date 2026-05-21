import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { qualityApi } from '../api/qualityApi';
import type { CreateQualityCertData, CertFilterParams } from '../types';

export function useCerts(params?: CertFilterParams) {
  return useQuery({
    queryKey: ['quality-certs', params],
    queryFn: () => qualityApi.getCerts(params),
  });
}

export function useCert(id: number) {
  return useQuery({
    queryKey: ['quality-cert', id],
    queryFn: () => qualityApi.getCert(id),
    enabled: !!id,
  });
}

export function useCreateCert() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateQualityCertData) => qualityApi.createCert(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['quality-certs'] });
    },
  });
}

export function useUpdateCert(id: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: Partial<CreateQualityCertData>) => qualityApi.updateCert(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['quality-certs'] });
      qc.invalidateQueries({ queryKey: ['quality-cert', id] });
    },
  });
}

export function useDeleteCert() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => qualityApi.deleteCert(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['quality-certs'] });
    },
  });
}

export function useGrades() {
  return useQuery({
    queryKey: ['quality-grades'],
    queryFn: () => qualityApi.getGrades(),
  });
}

export function useAttachments(certId: number) {
  return useQuery({
    queryKey: ['quality-attachments', certId],
    queryFn: () => qualityApi.getAttachments(certId),
    enabled: !!certId,
  });
}

export function useCreateAttachment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: FormData) => qualityApi.createAttachment(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['quality-attachments'] });
    },
  });
}

export function useDeleteAttachment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => qualityApi.deleteAttachment(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['quality-attachments'] });
    },
  });
}
