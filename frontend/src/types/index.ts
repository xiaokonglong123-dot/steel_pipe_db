export interface ApiResponse<T> {
  success: boolean;
  data: T;
}

export interface PaginatedData<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export interface PaginatedResponse<T> {
  success: boolean;
  data: PaginatedData<T>;
}

export interface UserInfo {
  id: number;
  username: string;
  display_name: string;
  role: string;
  email?: string | null;
  phone?: string | null;
}

export interface SeamlessPipe {
  id: number;
  pipe_number: string;
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
  location_id?: number;
  status: string;
  notes?: string;
  created_at: string;
  updated_at: string;
}

export interface ScreenPipe {
  id: number;
  pipe_number: string;
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
  location_id?: number;
  status: string;
  notes?: string;
  created_at: string;
  updated_at: string;
}
