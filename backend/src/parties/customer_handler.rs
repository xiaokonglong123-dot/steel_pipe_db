use serde::Deserialize;

use crate::dto::customer_dto::{
    CreateCustomerRequest, CustomerFilterParams, UpdateCustomerRequest,
};
use crate::models::customer::Customer;
use crate::parties::customer_service::CustomerService;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

crate::macros::party_handler! {
    service: CustomerService,
    model: Customer,
    create_dto: CreateCustomerRequest,
    update_dto: UpdateCustomerRequest,
    filter: CustomerFilterParams,
    list_fn: list_customers_handler,
    create_fn: create_customer_handler,
    get_fn: get_customer_handler,
    update_fn: update_customer_handler,
    delete_fn: delete_customer_handler,
    search_fn: search_customers_handler,
    active_fn: list_active_customers_handler
}
