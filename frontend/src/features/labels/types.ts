export interface PipeLabel {
  pipe_number: string;
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  length?: number;
  heat_number?: string;
  serial_number?: string;
  cert_number?: string;
  manufacturer?: string;
  production_date?: string;
  location?: string;
  status: string;
}

export interface BatchLabelRequest {
  pipe_ids: number[];
  pipe_type: string;
}

export interface ShippingLabelRequest {
  order_type: string;
  order_id: number;
}

export interface LabelData {
  label_id: string;
  content: string;
  format: string;
}
