use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")]
pub enum PipeCategory {
    #[serde(rename = "seamless")]
    Seamless,
    #[serde(rename = "screen")]
    Screen,
}

impl PipeCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            PipeCategory::Seamless => "seamless",
            PipeCategory::Screen => "screen",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")]
pub enum PipeStatus {
    #[serde(rename = "in_stock")]
    InStock,
    #[serde(rename = "outbound")]
    Outbound,
    #[serde(rename = "scrapped")]
    Scrapped,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")]
pub enum InboundType {
    #[serde(rename = "purchase")]
    Purchase,
    #[serde(rename = "return")]
    Return,
    #[serde(rename = "transfer")]
    Transfer,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")]
pub enum OutboundType {
    #[serde(rename = "sales")]
    Sales,
    #[serde(rename = "scrap")]
    Scrap,
    #[serde(rename = "transfer")]
    Transfer,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")]
pub enum CertResult {
    #[serde(rename = "pass")]
    Pass,
    #[serde(rename = "fail")]
    Fail,
    #[serde(rename = "pending")]
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")]
pub enum OrderStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "approved")]
    Approved,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Draft => "draft",
            OrderStatus::Pending => "pending",
            OrderStatus::Approved => "approved",
            OrderStatus::Completed => "completed",
            OrderStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")]
pub enum ContractStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "terminated")]
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")]
pub enum ContractType {
    #[serde(rename = "sales")]
    Sales,
    #[serde(rename = "purchase")]
    Purchase,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")]
pub enum UnitSystem {
    #[serde(rename = "metric")]
    Metric,
    #[serde(rename = "imperial")]
    Imperial,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub display_name: String,
    pub role: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SeamlessPipe {
    pub id: String,
    pub pipe_number: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub length: f64,
    pub weight: f64,
    pub connection_type: Option<String>,
    pub heat_number: Option<String>,
    pub production_date: Option<String>,
    pub status: String,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScreenPipe {
    pub id: String,
    pub pipe_number: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub length: f64,
    pub weight: f64,
    pub screen_type: String,
    pub slot_width: Option<f64>,
    pub open_area: Option<f64>,
    pub connection_type: Option<String>,
    pub heat_number: Option<String>,
    pub production_date: Option<String>,
    pub status: String,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InboundRecord {
    pub id: String,
    pub inbound_no: String,
    pub inbound_type: String,
    pub supplier_id: Option<String>,
    pub order_id: Option<String>,
    pub operator_id: String,
    pub total_items: i32,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InboundItem {
    pub id: String,
    pub inbound_id: String,
    pub pipe_type: String,
    pub pipe_id: String,
    pub confirmed: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OutboundRecord {
    pub id: String,
    pub outbound_no: String,
    pub outbound_type: String,
    pub customer_id: Option<String>,
    pub order_id: Option<String>,
    pub operator_id: String,
    pub total_items: i32,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OutboundItem {
    pub id: String,
    pub outbound_id: String,
    pub pipe_type: String,
    pub pipe_id: String,
    pub confirmed: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QualityCert {
    pub id: String,
    pub cert_no: String,
    pub pipe_type: String,
    pub pipe_id: String,
    pub inspect_date: String,
    pub inspector: String,
    pub agency: Option<String>,
    pub result: String,
    pub items_json: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PipeAttachment {
    pub id: String,
    pub pipe_type: String,
    pub pipe_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub uploaded_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Supplier {
    pub id: String,
    pub name: String,
    pub contact_person: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub cert_info: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Customer {
    pub id: String,
    pub name: String,
    pub contact_person: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PurchaseOrder {
    pub id: String,
    pub order_no: String,
    pub supplier_id: String,
    pub status: String,
    pub total_amount: f64,
    pub notes: Option<String>,
    pub operator_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PurchaseOrderItem {
    pub id: String,
    pub order_id: String,
    pub pipe_type: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub quantity: i32,
    pub received_quantity: i32,
    pub unit_price: f64,
    pub subtotal: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesOrder {
    pub id: String,
    pub order_no: String,
    pub customer_id: String,
    pub status: String,
    pub total_amount: f64,
    pub notes: Option<String>,
    pub operator_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesOrderItem {
    pub id: String,
    pub order_id: String,
    pub pipe_type: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub quantity: i32,
    pub delivered_quantity: i32,
    pub unit_price: f64,
    pub subtotal: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Contract {
    pub id: String,
    pub contract_no: String,
    pub contract_type: String,
    pub party_id: String,
    pub total_amount: f64,
    pub status: String,
    pub sign_date: Option<String>,
    pub effective_date: Option<String>,
    pub expiry_date: Option<String>,
    pub notes: Option<String>,
    pub operator_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContractItem {
    pub id: String,
    pub contract_id: String,
    pub description: String,
    pub spec: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
    pub amount: f64,
    pub delivery_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContractPayment {
    pub id: String,
    pub contract_id: String,
    pub stage: String,
    pub amount: f64,
    pub due_date: Option<String>,
    pub paid: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Api5ctGradeRef {
    pub grade: String,
    pub group_name: String,
    pub min_yield_strength: f64,
    pub max_yield_strength: f64,
    pub min_tensile_strength: f64,
    pub hardness_max: Option<f64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LabelTemplate {
    pub id: String,
    pub name: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub config_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryCheck {
    pub id: String,
    pub check_no: String,
    pub check_type: String,
    pub operator_id: String,
    pub total_expected: i32,
    pub total_confirmed: i32,
    pub total_missing: i32,
    pub notes: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryCheckItem {
    pub id: String,
    pub check_id: String,
    pub pipe_type: String,
    pub pipe_id: String,
    pub expected: bool,
    pub confirmed: bool,
    pub notes: Option<String>,
}
