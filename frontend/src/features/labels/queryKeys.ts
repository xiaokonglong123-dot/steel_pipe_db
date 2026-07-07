export const labelQueryKeys = {
  pipe: {
    all: ['pipe-label'] as const,
    detail: (pipeType: string, pipeId: number) => [...labelQueryKeys.pipe.all, pipeType, pipeId] as const,
  },
  quality: {
    detail: (certId: number) => ['quality-label', certId] as const,
  },
  shipping: {
    all: ['shipping-label'] as const,
  },
};
