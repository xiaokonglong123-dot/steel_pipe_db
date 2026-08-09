use serde::Deserialize;

use crate::dto::supplier_dto::{
    CreateSupplierRequest, SupplierFilterParams, UpdateSupplierRequest,
};
use crate::models::supplier::Supplier;
use crate::parties::supplier_service::SupplierService;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

crate::macros::party_handler! {
    service: SupplierService,
    model: Supplier,
    create_dto: CreateSupplierRequest,
    update_dto: UpdateSupplierRequest,
    filter: SupplierFilterParams,
    list_fn: list_suppliers_handler,
    create_fn: create_supplier_handler,
    get_fn: get_supplier_handler,
    update_fn: update_supplier_handler,
    delete_fn: delete_supplier_handler,
    search_fn: search_suppliers_handler,
    active_fn: list_active_suppliers_handler
}
