export interface PurchaseOrderItem {
  id: number;
  order_id: number;
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  quantity: number;
  received_quantity: number;
  unit_price?: number;
  total_price?: number;
  notes?: string;
  created_at: string;
}

export interface PurchaseOrder {
  id: number;
  order_no: string;
  supplier_id: number;
  order_date: string;
  status: string;
  total_amount?: number;
  notes?: string;
  created_by?: number;
  created_at: string;
  updated_at: string;
  deleted_at?: string;
}

export interface CreatePurchaseOrderItem {
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  quantity: number;
  unit_price?: number;
  notes?: string;
}

export interface CreatePurchaseOrderData {
  order_no?: string;
  supplier_id: number;
  order_date: string;
  notes?: string;
  items: CreatePurchaseOrderItem[];
}

export interface PurchaseOrderFilterParams {
  page?: number;
  page_size?: number;
  status?: string;
  supplier_id?: number;
  q?: string;
  sort_by?: string;
  sort_order?: string;
}

export interface PurchaseOrderStatusTransitionRequest {
  status: string;
}

export interface ApproveOrderRequest {
  notes?: string;
}

export interface RejectOrderRequest {
  reason: string;
}

export interface LinkInboundRequest {
  inbound_record_id: number;
}
