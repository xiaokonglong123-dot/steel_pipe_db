// 质量管理 API — 质量证书 CRUD + 钢级参考 + 附件上传
import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import type { QualityCert, CreateQualityCertData, CertFilterParams, GradeRef, Attachment } from '../types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { z } from 'zod';
import { qualityCertSchema, gradeRefSchema, attachmentSchema } from '@/zod-schemas/quality';

export const qualityApi = {
  getCerts: async (params?: CertFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<QualityCert>>('/quality/certs', { params });
    return validateResponse(paginatedDataSchema(qualityCertSchema), res.data.data);
  },

  getCert: async (id: number) => {
    const res = await apiClient.get<ApiResponse<QualityCert>>(`/quality/certs/${id}`);
    return validateResponse(qualityCertSchema, res.data.data);
  },

  createCert: async (data: CreateQualityCertData) => {
    const res = await apiClient.post<ApiResponse<QualityCert>>('/quality/certs', data);
    return validateResponse(qualityCertSchema, res.data.data);
  },

  updateCert: async (id: number, data: Partial<CreateQualityCertData>) => {
    const res = await apiClient.put<ApiResponse<QualityCert>>(`/quality/certs/${id}`, data);
    return validateResponse(qualityCertSchema, res.data.data);
  },

  deleteCert: async (id: number) => {
    await apiClient.delete(`/quality/certs/${id}`);
  },

  // 获取 API 5CT 钢级参考数据列表
  getGrades: async () => {
    const res = await apiClient.get<ApiResponse<GradeRef[]>>('/quality/grades');
    return validateResponse(z.array(gradeRefSchema), res.data.data);
  },

  getGradeByQuery: async (params: { grade: string }) => {
    const res = await apiClient.get<ApiResponse<GradeRef>>('/quality/grades/query', { params });
    return validateResponse(gradeRefSchema, res.data.data);
  },

  // 上传证书附件（PDF/图片），使用 FormData + multipart
  createAttachment: async (data: FormData) => {
    const res = await apiClient.post<ApiResponse<Attachment>>('/quality/attachments', data, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
    return validateResponse(attachmentSchema, res.data.data);
  },

  getAttachments: async (cert_id: number) => {
    const res = await apiClient.get<ApiResponse<Attachment[]>>('/quality/attachments', { params: { cert_id } });
    return validateResponse(z.array(attachmentSchema), res.data.data);
  },

  deleteAttachment: async (id: number) => {
    await apiClient.delete(`/quality/attachments/${id}`);
  },
};
