export interface QualityCert {
  id: number;
  cert_number: string;
  batch_number?: string;
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  length?: number;
  quantity: number;
  heat_number?: string;
  manufacturer?: string;
  production_date?: string;
  test_pressure?: number;
  yield_strength?: number;
  tensile_strength?: number;
  elongation?: number;
  hardness?: number;
  inspection_standard?: string;
  inspector?: string;
  cert_date?: string;
  status: string;
  notes?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateQualityCertData {
  cert_number: string;
  batch_number?: string;
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  length?: number;
  quantity: number;
  heat_number?: string;
  manufacturer?: string;
  production_date?: string;
  test_pressure?: number;
  yield_strength?: number;
  tensile_strength?: number;
  elongation?: number;
  hardness?: number;
  inspection_standard?: string;
  inspector?: string;
  cert_date?: string;
  status: string;
  notes?: string;
}

export interface CertFilterParams {
  page?: number;
  page_size?: number;
  cert_number?: string;
  grade?: string;
  status?: string;
  q?: string;
}

export interface GradeRef {
  id: number;
  grade: string;
  od_range?: string;
  wt_range?: string;
  min_yield?: number;
  max_yield?: number;
  min_tensile?: number;
  min_elongation?: number;
  max_hardness?: number;
  standard?: string;
}

export interface Attachment {
  id: number;
  cert_id: number;
  file_name: string;
  file_type: string;
  file_url: string;
  uploaded_at: string;
}
