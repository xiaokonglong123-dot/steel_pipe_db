export interface SalesOrder {
  id: number;
  order_no: string;
  customer_id: number;
  order_date: string;
  status: string;
  total_amount?: number;
  notes?: string;
  created_by?: number;
  created_at: string;
  updated_at: string;
  deleted_at?: string;
}

export interface SalesOrderItem {
  id: number;
  order_id: number;
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  quantity: number;
  delivered_quantity: number;
  unit_price?: number;
  total_price?: number;
  notes?: string;
  created_at: string;
}

export interface CreateSalesOrderData {
  customer_id: number;
  order_date: string;
  notes?: string;
  items: CreateSalesOrderItemData[];
}

export interface CreateSalesOrderItemData {
  pipe_type: string;
  grade: string;
  od: number;
  wt: number;
  quantity: number;
  unit_price?: number;
  notes?: string;
}

export interface SalesOrderFilterParams {
  page?: number;
  page_size?: number;
  status?: string;
  customer_id?: number;
  q?: string;
  sort_by?: string;
  sort_order?: string;
}

export interface SalesOrderStatusTransitionRequest {
  status: string;
}

export interface UpdateSalesOrderItemData {
  quantity?: number;
  unit_price?: number;
  notes?: string;
}
