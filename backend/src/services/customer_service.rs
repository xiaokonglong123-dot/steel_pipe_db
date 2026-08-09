use crate::dto::customer_dto::{
    CreateCustomerRequest, CustomerFilterParams, UpdateCustomerRequest,
};
use crate::models::customer::Customer;
use crate::repositories::customer_repo::CustomerRepo;

crate::macros::party_service! {
    service_name: CustomerService,
    model: Customer,
    repo: CustomerRepo,
    create_dto: CreateCustomerRequest,
    update_dto: UpdateCustomerRequest,
    filter: CustomerFilterParams,
    code_field: customer_code,
    code_dup_error: CustomerCodeDuplicate,
    not_found_error: CustomerNotFound,
    prefix: "CUS-"
}
