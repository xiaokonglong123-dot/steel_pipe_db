use crate::dto::supplier_dto::{
    CreateSupplierRequest, SupplierFilterParams, UpdateSupplierRequest,
};
use crate::models::supplier::Supplier;
use crate::parties::supplier_repo::SupplierRepo;

crate::macros::party_service! {
    service_name: SupplierService,
    model: Supplier,
    repo: SupplierRepo,
    create_dto: CreateSupplierRequest,
    update_dto: UpdateSupplierRequest,
    filter: SupplierFilterParams,
    code_field: supplier_code,
    code_dup_error: SupplierCodeDuplicate,
    not_found_error: SupplierNotFound,
    prefix: "SUP-"
}
