// API 响应边界校验 — 在后端数据进入前端状态前做类型安全把关
// 避免后端字段变更/格式异常导致前端运行时崩溃
import { z } from 'zod';

export function validateResponse<T>(schema: z.ZodType<T>, data: unknown): T {
  const result = schema.safeParse(data);
  if (!result.success) {
    console.error('[API Validation Error]', result.error.issues);
    throw new Error('API response validation failed');
  }
  return result.data;
}

/**
 * Creates a zod schema for unwrapped paginated data: `PaginatedData<T>`.
 * Use with endpoints that return `PaginatedResponse<T>` → after `.data.data` unwrap.
 */
export function paginatedDataSchema<T extends z.ZodTypeAny>(itemSchema: T) {
  return z.object({
    items: z.array(itemSchema),
    total: z.number(),
    page: z.number(),
    page_size: z.number(),
    total_pages: z.number(),
  });
}
