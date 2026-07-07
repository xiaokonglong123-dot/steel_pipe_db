use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

pub fn to_decimal(v: f64) -> rust_decimal::Decimal {
    rust_decimal::Decimal::from_f64(v).unwrap_or_default()
}

pub fn to_decimal_opt(v: Option<f64>) -> Option<rust_decimal::Decimal> {
    v.and_then(rust_decimal::Decimal::from_f64)
}

pub fn from_decimal(d: rust_decimal::Decimal) -> f64 {
    d.to_f64().unwrap_or(0.0)
}

pub fn from_decimal_opt(d: Option<rust_decimal::Decimal>) -> Option<f64> {
    d.and_then(|d| d.to_f64())
}
