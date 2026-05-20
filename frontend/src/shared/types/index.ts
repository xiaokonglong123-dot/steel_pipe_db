export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  meta?: PageMeta;
  request_id: string;
}

export interface ApiListResponse<T> {
  success: boolean;
  data: T[];
  meta?: PageMeta;
  request_id: string;
}

export interface PageMeta {
  page: number;
  page_size: number;
  total: number;
  total_pages: number;
}

export interface PipeSpec {
  grade: string;
  od: number;
  wt: number;
}

export type PipeCategory = 'seamless' | 'screen';
export type PipeStatus = 'in_stock' | 'outbound' | 'scrapped';
export type OrderStatus = 'draft' | 'pending' | 'approved' | 'completed' | 'cancelled';
export type CertResult = 'pass' | 'fail' | 'pending';
