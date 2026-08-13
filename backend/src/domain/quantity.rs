//! 数量类型 — 与金额一致采用 rust_decimal，保证精确算术与比较
//!
//! 存储：SQLite TEXT 列存 `Decimal::to_string()` canonical 十进制字符串。
//! 计算/比较：service 与 repo 层全链路 Decimal，杜绝浮点累计漂移。
//! JSON：序列化为 JSON number（前端类型保持 number，兼容现有页面）；
//!       反序列化接受 number 或 string。

use serde::{Deserialize, Deserializer, Serializer};
use std::str::FromStr;

use rust_decimal::Decimal;

pub type QuantityDec = Decimal;

/// 解析数量，失败返回校验错误
pub fn parse_qty(s: &str) -> Result<Decimal, crate::AppError> {
    Decimal::from_str(s).map_err(|_| crate::AppError::validation(format!("无效的数量: {s}")))
}

/// JSON 序列化：Decimal → JSON number（保留到前端 number）
pub fn serialize<S: Serializer>(v: &Decimal, s: S) -> Result<S::Ok, S::Error> {
    // f64 表达在业务数量（件/吨/米）范围内足够，且前端为 number
    let f = rust_decimal::prelude::ToPrimitive::to_f64(v)
        .ok_or_else(|| serde::ser::Error::custom("数量超出 f64 范围"))?;
    s.serialize_f64(f)
}

/// JSON 反序列化：number 或 string → Decimal
pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Decimal, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Repr {
        Str(String),
        Num(f64),
    }
    match Repr::deserialize(d)? {
        Repr::Str(s) => Decimal::from_str(&s)
            .map_err(|e| serde::de::Error::custom(format!("invalid quantity string: {e}"))),
        Repr::Num(n) => Decimal::from_f64_retain(n)
            .ok_or_else(|| serde::de::Error::custom("invalid quantity number")),
    }
}


/// 序列化 String 存储的数量（SQLite TEXT）为 JSON number。
/// 用于 Row struct 中从 sqlx 读出的 String 数量字段，保持 API 输出为 number 的既有契约。
pub fn serialize_qty_str<S: Serializer>(v: &str, ser: S) -> Result<S::Ok, S::Error> {
    let dec = Decimal::from_str(v)
        .map_err(|_| serde::ser::Error::custom(format!("invalid quantity string: {v}")))?;
    let f = rust_decimal::prelude::ToPrimitive::to_f64(&dec)
        .ok_or_else(|| serde::ser::Error::custom("quantity out of f64 range"))?;
    ser.serialize_f64(f)
}


/// 序列化 Option<String> 数量的 JSON number（None -> null）。
pub fn serialize_opt_qty_str<S: Serializer>(v: &Option<String>, ser: S) -> Result<S::Ok, S::Error> {
    match v {
        Some(s) => serialize_qty_str(s, ser),
        None => ser.serialize_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_qty_roundtrip() {
        let q = parse_qty("12.5").unwrap();
        assert_eq!(q.to_string(), "12.5");
    }

    #[test]
    fn parse_qty_rejects_garbage() {
        assert!(parse_qty("abc").is_err());
    }

    #[test]
    fn qty_math_is_exact() {
        let a = parse_qty("0.1").unwrap();
        let b = parse_qty("0.2").unwrap();
        assert_eq!((a + b).to_string(), "0.3"); // f64 会得到 0.30000000000000004
    }
}
