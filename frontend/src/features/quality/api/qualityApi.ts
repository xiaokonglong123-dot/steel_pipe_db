// 质量管理 API — 质量证书 CRUD + 钢级参考 + 附件上传
import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';
import type { QualityCert, CreateQualityCertData, CertFilterParams, GradeRef, PipeAttachment } from '../types';
import { validateResponse, paginatedDataSchema } from '@/lib/validateResponse';
import { z } from 'zod';
import { qualityCertSchema, gradeRefSchema, pipeAttachmentSchema } from '@/zod-schemas/quality';

export const qualityApi = {
  getCerts: async (params?: CertFilterParams) => {
    const res = await apiClient.get<PaginatedResponse<QualityCert>>(
      '/quality/certs',
      params as Record<string, unknown>,
    );
    return validateResponse(paginatedDataSchema(qualityCertSchema), res.data);
  },

  getCert: async (id: number) => {
    const res = await apiClient.get<ApiResponse<QualityCert>>(`/quality/certs/${id}`);
    return validateResponse(qualityCertSchema, res.data);
  },

  createCert: async (data: CreateQualityCertData) => {
    const res = await apiClient.post<ApiResponse<QualityCert>>('/quality/certs', data);
    return validateResponse(qualityCertSchema, res.data);
  },

  updateCert: async (id: number, data: Partial<CreateQualityCertData>) => {
    const res = await apiClient.put<ApiResponse<QualityCert>>(`/quality/certs/${id}`, data);
    return validateResponse(qualityCertSchema, res.data);
  },

  deleteCert: async (id: number) => {
    await apiClient.delete(`/quality/certs/${id}`);
  },

  // 获取 API 5CT 钢级参考数据列表
  getGrades: async () => {
    const res = await apiClient.get<ApiResponse<GradeRef[]>>('/quality/grades');
    return validateResponse(z.array(gradeRefSchema), res.data);
  },

  getGradeByQuery: async (params: { grade: string }) => {
    const res = await apiClient.get<ApiResponse<GradeRef>>(
      '/quality/grades/query',
      params as Record<string, unknown>,
    );
    return validateResponse(gradeRefSchema, res.data);
  },

  // 上传证书附件（PDF/图片），使用 FormData + multipart
  createAttachment: async (data: FormData) => {
    const res = await apiClient.postFormData<ApiResponse<PipeAttachment>>(
      '/quality/attachments',
      data,
    );
    return validateResponse(pipeAttachmentSchema, res.data);
  },

  getAttachments: async (pipe_id: number) => {
    const res = await apiClient.get<ApiResponse<PipeAttachment[]>>(
      '/quality/attachments',
      { pipe_id },
    );
    return validateResponse(z.array(pipeAttachmentSchema), res.data);
  },

  deleteAttachment: async (id: number) => {
    await apiClient.delete(`/quality/attachments/${id}`);
  },
};
