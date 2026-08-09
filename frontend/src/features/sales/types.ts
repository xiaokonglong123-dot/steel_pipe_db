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
  item_id: number;
  quantity: number;
  delivered_quantity: number;
  unit_price?: number;
  total_price?: number;
  notes?: string;
  created_at: string;
  /** Display-only fields (not returned by the backend; used for optimistic rows). */
  sku?: string;
  name?: string;
}

export interface CreateSalesOrderData {
  customer_id: number;
  order_date: string;
  notes?: string;
  items: CreateSalesOrderItemData[];
}

export interface CreateSalesOrderItemData {
  /** Item master ID (`items.id`). */
  item_id: number;
  /** Display-only: SKU / name of the picked item. */
  sku?: string;
  name?: string;
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

export interface ApproveSalesOrderRequest {
  notes?: string;
}

export interface RejectSalesOrderRequest {
  reason: string;
}

export interface LinkOutboundRequest {
  outbound_record_id: number;
}

export interface UpdateSalesOrderItemData {
  quantity?: number;
  unit_price?: number;
  notes?: string;
}
