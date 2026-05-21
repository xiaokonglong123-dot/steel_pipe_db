import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import type { QualityCert, CreateQualityCertData, CertFilterParams, GradeRef, Attachment } from '../types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { z } from 'zod';
import { qualityCertSchema, gradeRefSchema, attachmentSchema } from '@/zod-schemas/quality';

export const qualityApi = {
  getCerts: async (params?: CertFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<QualityCert>>('/quality/certs', { params });
    return validateResponse(res.data.data, paginatedDataSchema(qualityCertSchema));
  },

  getCert: async (id: number) => {
    const res = await apiClient.get<ApiResponse<QualityCert>>(`/quality/certs/${id}`);
    return validateResponse(res.data.data, qualityCertSchema);
  },

  createCert: async (data: CreateQualityCertData) => {
    const res = await apiClient.post<ApiResponse<QualityCert>>('/quality/certs', data);
    return validateResponse(res.data.data, qualityCertSchema);
  },

  updateCert: async (id: number, data: Partial<CreateQualityCertData>) => {
    const res = await apiClient.put<ApiResponse<QualityCert>>(`/quality/certs/${id}`, data);
    return validateResponse(res.data.data, qualityCertSchema);
  },

  deleteCert: async (id: number) => {
    await apiClient.delete(`/quality/certs/${id}`);
  },

  getGrades: async () => {
    const res = await apiClient.get<ApiResponse<GradeRef[]>>('/quality/grades');
    return validateResponse(res.data.data, z.array(gradeRefSchema));
  },

  getGradeByQuery: async (params: { grade: string }) => {
    const res = await apiClient.get<ApiResponse<GradeRef>>('/quality/grades/query', { params });
    return validateResponse(res.data.data, gradeRefSchema);
  },

  createAttachment: async (data: FormData) => {
    const res = await apiClient.post<ApiResponse<Attachment>>('/quality/attachments', data, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
    return validateResponse(res.data.data, attachmentSchema);
  },

  getAttachments: async (cert_id: number) => {
    const res = await apiClient.get<ApiResponse<Attachment[]>>('/quality/attachments', { params: { cert_id } });
    return validateResponse(res.data.data, z.array(attachmentSchema));
  },

  deleteAttachment: async (id: number) => {
    await apiClient.delete(`/quality/attachments/${id}`);
  },
};
