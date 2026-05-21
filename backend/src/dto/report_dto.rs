use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OrderReportQuery {
    pub r#type: Option<String>,
    pub period: Option<String>,
}
