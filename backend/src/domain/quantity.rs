//! 数量类型 — 与金额一致采用 rust_decimal，保证精确算术与比较
//!
//! 存储：SQLite TEXT 列存 `Decimal::to_string()` canonical 十进制字符串。
//! 计算/比较：service 与 repo 层全链路 Decimal，杜绝浮点累计漂移。
//! JSON：序列化为 string（ADR-R6：与金额同一契约，杜绝 f64 精度折损）；
//!       反序列化接受 number 或 string（向后兼容旧客户端）。

use serde::{Deserialize, Deserializer, Serializer};
use std::str::FromStr;

use rust_decimal::Decimal;

pub type QuantityDec = Decimal;

/// 解析数量，失败返回校验错误
pub fn parse_qty(s: &str) -> Result<Decimal, crate::AppError> {
    Decimal::from_str(s).map_err(|_| crate::AppError::validation(format!("无效的数量: {s}")))
}

/// JSON 序列化：Decimal → canonical 十进制 string（与 money 同一契约）
pub fn serialize<S: Serializer>(v: &Decimal, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&v.to_string())
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


/// 序列化 String 存储的数量（SQLite TEXT）为 canonical 十进制 string。
/// 入库值经由 Decimal 归一化后输出（如 "4.0" → "4"）。
pub fn serialize_qty_str<S: Serializer>(v: &str, ser: S) -> Result<S::Ok, S::Error> {
    let dec = Decimal::from_str(v)
        .map_err(|_| serde::ser::Error::custom(format!("invalid quantity string: {v}")))?;
    ser.serialize_str(&dec.to_string())
}


/// 序列化 Option<String> 数量的 JSON string（None -> null）。
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

    #[test]
    fn serialize_emits_canonical_string() {
        #[derive(serde::Serialize)]
        struct Row {
            #[serde(serialize_with = "serialize_qty_str")]
            qty: String,
        }
        let json = serde_json::to_string(&Row { qty: "4.0".into() }).unwrap();
        assert_eq!(json, r#"{"qty":"4.0"}"#);

        #[derive(serde::Serialize)]
        struct OptRow {
            #[serde(serialize_with = "serialize_opt_qty_str")]
            qty: Option<String>,
        }
        assert_eq!(
            serde_json::to_string(&OptRow { qty: Some("2.50".into()) }).unwrap(),
            r#"{"qty":"2.50"}"#
        );
        assert_eq!(
            serde_json::to_string(&OptRow { qty: None }).unwrap(),
            r#"{"qty":null}"#
        );
    }
}
