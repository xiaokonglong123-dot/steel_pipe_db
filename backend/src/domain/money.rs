//! 金额类型 — 全链路 rust_decimal（对齐 detailed-design §3 ADR-002 决议）
//!
//! 存储：SQLite TEXT 列存 `Decimal::to_string()` canonical 十进制字符串。
//! 读：从 String parse 为 Decimal。
//! 计算：service 层全链路 Decimal。
//! JSON：序列化为 JSON number（保留精度）。

use serde::{Deserialize, Deserializer, Serializer};
use std::str::FromStr;

use rust_decimal::Decimal;

pub type MoneyDec = Decimal;

/// 解析 Decimal，失败返回校验错误
pub fn parse_amount(s: &str) -> Result<Decimal, crate::AppError> {
    Decimal::from_str(s).map_err(|_| crate::AppError::validation(format!("无效的金额: {s}")))
}

/// JSON 序列化：Decimal → 字符串（保留 scale，避免 JS number 精度丢失）
pub fn serialize<S: Serializer>(v: &Decimal, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&v.to_string())
}

/// JSON 反序列化：字符串 或 数字 → Decimal
pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Decimal, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Repr {
        Str(String),
        Num(f64),
    }
    match Repr::deserialize(d)? {
        Repr::Str(s) => Decimal::from_str(&s)
            .map_err(|e| serde::de::Error::custom(format!("invalid decimal: {e}"))),
        Repr::Num(n) => Decimal::from_f64_retain(n)
            .ok_or_else(|| serde::de::Error::custom("invalid decimal number")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_amount_roundtrip() {
        let d = parse_amount("1234.56").unwrap();
        assert_eq!(d.to_string(), "1234.56");
    }

    #[test]
    fn parse_amount_rejects_garbage() {
        assert!(parse_amount("abc").is_err());
    }

    #[test]
    fn decimal_math_is_exact() {
        let a = parse_amount("0.1").unwrap();
        let b = parse_amount("0.2").unwrap();
        assert_eq!((a + b).to_string(), "0.3"); // 浮点会得到 0.30000000000000004
    }
}
