use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt;

/// Pipe number value object - ensures valid format and uniqueness
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipeNumber(String);

impl PipeNumber {
    pub fn new(value: String) -> Result<Self, crate::error::AppError> {
        if value.is_empty() {
            return Err(crate::error::AppError::Validation("Pipe number cannot be empty".into()));
        }
        // Validate format: prefix-grade-odxwt-serial
        Ok(Self(value))
    }

    pub fn generate(prefix: &str, grade: &str, od: f64, wt: f64) -> Self {
        let serial = Uuid::new_v4().to_string();
        let short_serial = &serial[..8];
        Self(format!("{}-{}-{}x{}-{}", prefix, grade, od, wt, short_serial))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PipeNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<PipeNumber> for String {
    fn from(pipe_number: PipeNumber) -> Self {
        pipe_number.0
    }
}

/// Heat number value object - tracks steel heat/batch
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HeatNumber(String);

impl HeatNumber {
    pub fn new(value: String) -> Result<Self, crate::error::AppError> {
        if value.is_empty() {
            return Err(crate::error::AppError::Validation("Heat number cannot be empty".into()));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HeatNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Money value object - handles currency with proper precision
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Money {
    amount: i64, // Stored in smallest unit (cents/fen)
    currency: Currency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    CNY,
    USD,
}

impl Money {
    pub fn new_cny(yuan: f64) -> Self {
        Self {
            amount: (yuan * 100.0).round() as i64,
            currency: Currency::CNY,
        }
    }

    pub fn new_usd(dollars: f64) -> Self {
        Self {
            amount: (dollars * 100.0).round() as i64,
            currency: Currency::USD,
        }
    }

    pub fn yuan(&self) -> f64 {
        self.amount as f64 / 100.0
    }

    pub fn dollars(&self) -> f64 {
        self.amount as f64 / 100.0
    }

    pub fn add(&self, other: Money) -> Result<Money, crate::error::AppError> {
        if self.currency != other.currency {
            return Err(crate::error::AppError::Validation("Currency mismatch".into()));
        }
        Ok(Money {
            amount: self.amount + other.amount,
            currency: self.currency,
        })
    }

    pub fn subtract(&self, other: Money) -> Result<Money, crate::error::AppError> {
        if self.currency != other.currency {
            return Err(crate::error::AppError::Validation("Currency mismatch".into()));
        }
        Ok(Money {
            amount: self.amount - other.amount,
            currency: self.currency,
        })
    }
}

/// Quantity value object - handles integer quantities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quantity(i64);

impl Quantity {
    pub fn new(value: i64) -> Result<Self, crate::error::AppError> {
        if value < 0 {
            return Err(crate::error::AppError::Validation("Quantity cannot be negative".into()));
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> i64 {
        self.0
    }
}

impl std::ops::Add for Quantity {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::Sub for Quantity {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl From<i64> for Quantity {
    fn from(value: i64) -> Self {
        Self(value)
    }
}