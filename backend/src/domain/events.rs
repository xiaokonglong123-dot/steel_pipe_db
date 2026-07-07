use crate::domain::value_objects::{PipeNumber, HeatNumber};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Domain events - represent things that happened in the domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    /// Pipe was created
    PipeCreated {
        pipe_id: Uuid,
        pipe_number: PipeNumber,
    },
    /// Pipe was moved to a new location
    PipeMoved {
        pipe_id: Uuid,
        location_id: Uuid,
    },
    /// Pipe was shipped out
    PipeOutbound {
        pipe_id: Uuid,
    },
    /// Pipe was scrapped
    PipeScrapped {
        pipe_id: Uuid,
    },
    /// Inbound record created
    InboundCreated {
        inbound_id: i64,
        inbound_no: String,
    },
    /// Inbound approved
    InboundApproved {
        inbound_id: i64,
    },
    /// Inbound rejected
    InboundRejected {
        inbound_id: i64,
        reason: String,
    },
    /// Outbound record created
    OutboundCreated {
        outbound_id: i64,
        outbound_no: String,
    },
    /// Outbound approved
    OutboundApproved {
        outbound_id: i64,
    },
    /// Outbound rejected
    OutboundRejected {
        outbound_id: i64,
        reason: String,
    },
    /// Purchase order created
    PurchaseOrderCreated {
        order_id: i64,
        order_no: String,
    },
    /// Purchase order approved
    PurchaseOrderApproved {
        order_id: i64,
    },
    /// Purchase order rejected
    PurchaseOrderRejected {
        order_id: i64,
        reason: String,
    },
    /// Sales order created
    SalesOrderCreated {
        order_id: i64,
        order_no: String,
    },
    /// Sales order approved (with ATP check)
    SalesOrderApproved {
        order_id: i64,
    },
    /// Sales order rejected
    SalesOrderRejected {
        order_id: i64,
        reason: String,
    },
    /// Quality certificate created
    QualityCertCreated {
        cert_id: i64,
        cert_number: String,
    },
    /// Contract created
    ContractCreated {
        contract_id: i64,
        contract_no: String,
    },
    /// Inventory check created
    InventoryCheckCreated {
        check_id: i64,
    },
    /// Inventory check completed
    InventoryCheckCompleted {
        check_id: i64,
    },
    /// Location created
    LocationCreated {
        location_id: i64,
        full_code: String,
    },
    /// Supplier created
    SupplierCreated {
        supplier_id: i64,
        supplier_code: String,
    },
    /// Customer created
    CustomerCreated {
        customer_id: i64,
        customer_code: String,
    },
}

/// Event publisher trait - implemented by infrastructure
pub trait EventPublisher: Send + Sync {
    fn publish(&self, event: DomainEvent) -> impl std::future::Future<Output = Result<(), crate::error::AppError>> + Send;
}

/// No-op event publisher for testing
pub struct NoopEventPublisher;

impl EventPublisher for NoopEventPublisher {
    async fn publish(&self, _event: DomainEvent) -> Result<(), crate::error::AppError> {
        Ok(())
    }
}