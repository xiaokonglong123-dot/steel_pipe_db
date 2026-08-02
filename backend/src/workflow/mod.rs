//! Workflow domain — unified approval engine (definitions, instances, nodes,
//! delegation). Business modules consume it via `WorkflowService` and the
//! `/api/v1/workflows/*` endpoints; callback hooks (`callback_action`) are
//! consumed by the business modules themselves in a later phase.

pub mod handlers;
pub mod repos;
pub mod services;
