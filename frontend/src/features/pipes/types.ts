import type { SeamlessPipe, ScreenPipe } from '@/types';

export interface CreateSeamlessPipeData {
  pipe_number?: string;
  batch_number?: string;
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  length?: number;
  weight_per_unit?: number;
  end_type?: string;
  coupling_type?: string;
  coupling_od?: number;
  coupling_length?: number;
  heat_number?: string;
  serial_number?: string;
  manufacturer?: string;
  production_date?: string;
  cert_number?: string;
  notes?: string;
}

export interface CreateScreenPipeData {
  pipe_number?: string;
  batch_number?: string;
  screen_type: string;
  slot_size?: number;
  filtration_grade?: string;
  base_od: number;
  base_wt: number;
  base_grade: string;
  base_end_type?: string;
  length?: number;
  weight_per_unit?: number;
  heat_number?: string;
  serial_number?: string;
  manufacturer?: string;
  production_date?: string;
  cert_number?: string;
  notes?: string;
}

export interface PipeFilterParams {
  q?: string;
  grade?: string;
  pipe_type?: string;
  status?: string;
  od_min?: number;
  od_max?: number;
  wt_min?: number;
  wt_max?: number;
  location_id?: number;
  manufacturer?: string;
  page?: number;
  page_size?: number;
  sort_by?: string;
  sort_order?: string;
}

export type { SeamlessPipe, ScreenPipe };
