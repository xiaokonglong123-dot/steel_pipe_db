use chrono::Utc;
use std::str::FromStr;

use crate::domain::order::OrderStatus;
use crate::error::AppError;

pub fn generate_no(prefix: &str) -> String {
    let now = Utc::now();
    let date_str = now.format("%Y%m%d").to_string();
    let serial = uuid::Uuid::new_v4().to_string();
    let short_serial = &serial[..8];
    format!("{}-{}-{}", prefix, date_str, short_serial)
}

pub fn validate_status_transition(current: &str, target: &str) -> Result<(), AppError> {
    let current_status = OrderStatus::from_str(current)
        .map_err(|_| AppError::Validation(format!("Invalid current status: {}", current)))?;
    let target_status = OrderStatus::from_str(target)
        .map_err(|_| AppError::Validation(format!("Invalid target status: {}", target)))?;

    if !current_status.valid_transition(&target_status) {
        return Err(AppError::OrderCannotModify(format!(
            "Cannot transition from '{}' to '{}'",
            current, target
        )));
    }
    Ok(())
}
