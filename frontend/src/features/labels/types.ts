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
  pipe_ids: { pipe_type: string; pipe_id: number }[];
}

export interface ShippingLabelRequest {
  pipe_type: string;
  pipe_id: number;
  order_number?: string;
  customer_name?: string;
  destination?: string;
  po_number?: string;
  ship_date?: string;
}

export interface LabelData {
  label_id: string;
  content: string;
  format: string;
}
