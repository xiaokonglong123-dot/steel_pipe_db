export interface PurchaseOrderItem {
  id: number;
  purchase_order_id: number;
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  length?: number;
  quantity: number;
  unit_price: number;
  total_price: number;
  notes?: string;
}

export interface PurchaseOrder {
  id: number;
  order_number: string;
  supplier_id: number;
  supplier_name: string;
  order_date: string;
  expected_date?: string;
  status: string;
  total_amount: number;
  notes?: string;
  items: PurchaseOrderItem[];
  created_at: string;
  updated_at: string;
}

export interface CreatePurchaseOrderItem {
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  length?: number;
  quantity: number;
  unit_price: number;
  notes?: string;
}

export interface CreatePurchaseOrderData {
  supplier_id: number;
  order_date: string;
  expected_date?: string;
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
  notes?: string;
}
