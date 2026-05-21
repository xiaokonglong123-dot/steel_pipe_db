import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import type { QualityCert, CreateQualityCertData, CertFilterParams, GradeRef, Attachment } from '../types';

export const qualityApi = {
  getCerts: async (params?: CertFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<QualityCert>>('/quality/certs', { params });
    return res.data.data;
  },

  getCert: async (id: number) => {
    const res = await apiClient.get<ApiResponse<QualityCert>>(`/quality/certs/${id}`);
    return res.data.data;
  },

  createCert: async (data: CreateQualityCertData) => {
    const res = await apiClient.post<ApiResponse<QualityCert>>('/quality/certs', data);
    return res.data.data;
  },

  updateCert: async (id: number, data: Partial<CreateQualityCertData>) => {
    const res = await apiClient.put<ApiResponse<QualityCert>>(`/quality/certs/${id}`, data);
    return res.data.data;
  },

  deleteCert: async (id: number) => {
    await apiClient.delete(`/quality/certs/${id}`);
  },

  getGrades: async () => {
    const res = await apiClient.get<ApiResponse<GradeRef[]>>('/quality/grades');
    return res.data.data;
  },

  getGradeByQuery: async (params: { grade: string }) => {
    const res = await apiClient.get<ApiResponse<GradeRef>>('/quality/grades/query', { params });
    return res.data.data;
  },

  createAttachment: async (data: FormData) => {
    const res = await apiClient.post<ApiResponse<Attachment>>('/quality/attachments', data, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
    return res.data.data;
  },

  getAttachments: async (cert_id: number) => {
    const res = await apiClient.get<ApiResponse<Attachment[]>>('/quality/attachments', { params: { cert_id } });
    return res.data.data;
  },

  deleteAttachment: async (id: number) => {
    await apiClient.delete(`/quality/attachments/${id}`);
  },
};
