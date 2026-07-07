/**
 * Global TypeScript type definitions for the Steel Pipe DB frontend.
 *
 * These types mirror the backend response shapes (see backend/src/response.rs
 * and backend/src/models/). Field names use snake_case to match the JSON API
 * response — do NOT convert to camelCase or the API client will break.
 *
 * Convention: each feature module may also define local types in its own
 * `types/index.ts` for feature-specific request/response shapes.
 */

/** Standard API response envelope — matches backend `ApiResponse<T>`. */
export interface ApiResponse<T> {
  success: boolean;
  request_id: string;
  data: T;
}

/** Paginated data payload — matches backend `PaginatedData<T>`. */
export interface PaginatedData<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

/** Pagination metadata — matches backend `Meta` struct. */
export interface Meta {
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

/** Paginated API response envelope — matches backend `PaginatedResponse<T>`. */
export interface PaginatedResponse<T> {
  success: boolean;
  request_id: string;
  data: PaginatedData<T>;
  meta: Meta;
}

/** Authenticated user info — from `/api/v1/auth/me` and stored in authStore. */
export interface UserInfo {
  id: number;
  username: string;
  display_name: string;
  /** RBAC role: admin | warehouse | qc | sales */
  role: string;
  email?: string | null;
  phone?: string | null;
}

/** API 5CT seamless pipe (casing/tubing) — matches `seamless_pipes` table. */
export interface SeamlessPipe {
  id: number;
  pipe_number: string;
  batch_number?: string | null;
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  length?: number | null;
  weight_per_unit?: number | null;
  end_type?: string | null;
  coupling_type?: string | null;
  coupling_od?: number | null;
  coupling_length?: number | null;
  heat_number?: string | null;
  serial_number?: string | null;
  manufacturer?: string | null;
  production_date?: string | null;
  cert_number?: string | null;
  location_id?: number | null;
  status: string;
  notes?: string | null;
  created_at: string;
  updated_at: string;
  deleted_at?: string | null;
}

/** API 5CT screen pipe (slotted/wire-wrapped) — matches `screen_pipes` table. */
export interface ScreenPipe {
  id: number;
  pipe_number: string;
  batch_number?: string | null;
  screen_type: string;
  slot_size?: number | null;
  filtration_grade?: string | null;
  base_od: number;
  base_wt: number;
  base_grade: string;
  base_end_type?: string | null;
  length?: number | null;
  weight_per_unit?: number | null;
  heat_number?: string | null;
  serial_number?: string | null;
  manufacturer?: string | null;
  production_date?: string | null;
  cert_number?: string | null;
  location_id?: number | null;
  status: string;
  notes?: string | null;
  created_at: string;
  updated_at: string;
  deleted_at?: string | null;
}
