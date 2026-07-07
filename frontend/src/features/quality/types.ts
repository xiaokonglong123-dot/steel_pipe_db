export interface QualityCert {
  id: number;
  cert_number: string;
  pipe_type: string;
  pipe_id: number;
  cert_date?: string;
  result: string;
  inspector?: string;
  inspection_body?: string;
  notes?: string;
  created_at: string;
  updated_at: string;
  deleted_at?: string;
}

export interface PipeAttachment {
  id: number;
  pipe_type: string;
  pipe_id: number;
  file_name: string;
  file_path: string;
  file_size?: number;
  content_type?: string;
  uploaded_by?: number;
  created_at: string;
}

export interface CreateQualityCertData {
  cert_number: string;
  pipe_type: string;
  pipe_id: number;
  cert_date?: string;
  result: string;
  inspector?: string;
  inspection_body?: string;
  notes?: string;
}

export interface CertFilterParams {
  page?: number;
  page_size?: number;
  cert_number?: string;
  q?: string;
}

export interface GradeRef {
  id: number;
  grade: string;
  yield_strength_min?: number;
  yield_strength_max?: number;
  tensile_strength_min?: number;
  hardness_max?: string;
  carbon_content_max?: number;
  manganese_content_max?: number;
  phosphorus_content_max?: number;
  sulfur_content_max?: number;
  notes?: string;
}
