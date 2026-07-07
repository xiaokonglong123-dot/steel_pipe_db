import type { CertFilterParams } from './types';

export const qualityQueryKeys = {
  certs: {
    all: ['quality-certs'] as const,
    list: (params?: CertFilterParams) => [...qualityQueryKeys.certs.all, params] as const,
    detail: (id: number) => ['quality-cert', id] as const,
  },
  grades: () => ['quality-grades'] as const,
  attachments: {
    all: ['quality-attachments'] as const,
    list: (certId: number) => [...qualityQueryKeys.attachments.all, certId] as const,
  },
};
