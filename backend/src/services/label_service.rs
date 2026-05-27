use sqlx::SqlitePool;

use crate::dto::label_dto::{BatchLabelRequest, ShippingLabelRequest};
use crate::error::AppError;
use crate::repositories::label_repo::LabelRepo;

/// Label generation service — produces HTML barcode labels, QC tags, and shipping
/// labels for steel pipes. Renders seamless and screen pipe specs into printable
/// HTML documents.
pub struct LabelService;

impl LabelService {
    /// Generates a barcode label HTML for a single pipe. Queries the pipe data by
    /// `pipe_type` (`seamless`/`screen`) and renders a 4-inch spec label.
    ///
    /// # Errors
    /// - `AppError::PipeNotFound` — pipe ID doesn't exist
    /// - `AppError::Validation` — invalid pipe_type
    pub async fn generate_pipe_label(
        pool: &SqlitePool,
        pipe_type: &str,
        pipe_id: i64,
    ) -> Result<String, AppError> {
        match pipe_type {
            "seamless" => {
                let pipe = LabelRepo::find_seamless_pipe(pool, pipe_id)
                    .await
                    .map_err(AppError::from)?
                    .ok_or_else(|| AppError::PipeNotFound(format!("Seamless pipe id={}", pipe_id)))?;
                Ok(Self::seamless_barcode_html(&pipe))
            }
            "screen" => {
                let pipe = LabelRepo::find_screen_pipe(pool, pipe_id)
                    .await
                    .map_err(AppError::from)?
                    .ok_or_else(|| AppError::PipeNotFound(format!("Screen pipe id={}", pipe_id)))?;
                Ok(Self::screen_barcode_html(&pipe))
            }
            _ => Err(AppError::Validation(format!(
                "Invalid pipe_type '{}'. Must be 'seamless' or 'screen'",
                pipe_type
            ))),
        }
    }

    /// Batch-generates barcode label HTML for multiple pipes. Processes each pipe_id
    /// in the request and merges them into one continuous print-ready HTML doc
    /// (page break per label).
    ///
    /// # Errors
    /// - `AppError::PipeNotFound` — any pipe ID doesn't exist
    /// - `AppError::Validation` — any pipe_type is invalid
    pub async fn generate_batch_labels(
        pool: &SqlitePool,
        req: &BatchLabelRequest,
    ) -> Result<String, AppError> {
        let mut labels = Vec::new();
        for pid in &req.pipe_ids {
            let label = match pid.pipe_type.as_str() {
                "seamless" => {
                    let pipe = LabelRepo::find_seamless_pipe(pool, pid.pipe_id)
                        .await
                        .map_err(AppError::from)?
                        .ok_or_else(|| {
                            AppError::PipeNotFound(format!(
                                "Seamless pipe id={}",
                                pid.pipe_id
                            ))
                        })?;
                    Self::seamless_barcode_html(&pipe)
                }
                "screen" => {
                    let pipe = LabelRepo::find_screen_pipe(pool, pid.pipe_id)
                        .await
                        .map_err(AppError::from)?
                        .ok_or_else(|| {
                            AppError::PipeNotFound(format!("Screen pipe id={}", pid.pipe_id))
                        })?;
                    Self::screen_barcode_html(&pipe)
                }
                _ => {
                    return Err(AppError::Validation(format!(
                        "Invalid pipe_type '{}' at pipe_id={}",
                        pid.pipe_type, pid.pipe_id
                    )));
                }
            };
            labels.push(label);
        }
        Ok(Self::batch_html(&labels))
    }

    /// Generates a QC tag HTML. Queries the quality cert by ID and its linked pipe,
    /// renders a status tag with cert number, grade, test results, and inspector.
    ///
    /// # Errors
    /// - `AppError::QualityCertNotFound` — cert ID doesn't exist
    /// - `AppError::PipeNotFound` — linked pipe doesn't exist
    pub async fn generate_quality_tag(
        pool: &SqlitePool,
        cert_id: i64,
    ) -> Result<String, AppError> {
        let cert = LabelRepo::find_quality_cert(pool, cert_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::QualityCertNotFound(format!("Cert id={}", cert_id)))?;

        let (pipe_number, grade) = match cert.pipe_type.as_str() {
            "seamless" => {
                let pipe = LabelRepo::find_seamless_pipe(pool, cert.pipe_id)
                    .await
                    .map_err(AppError::from)?
                    .ok_or_else(|| {
                        AppError::PipeNotFound(format!("Seamless pipe id={}", cert.pipe_id))
                    })?;
                (pipe.pipe_number, pipe.grade)
            }
            "screen" => {
                let pipe = LabelRepo::find_screen_pipe(pool, cert.pipe_id)
                    .await
                    .map_err(AppError::from)?
                    .ok_or_else(|| {
                        AppError::PipeNotFound(format!("Screen pipe id={}", cert.pipe_id))
                    })?;
                (pipe.pipe_number, pipe.base_grade)
            }
            _ => {
                return Err(AppError::Validation(format!(
                    "Unknown pipe_type '{}' on cert id={}",
                    cert.pipe_type, cert.id
                )));
            }
        };

        Ok(Self::quality_tag_html(&cert, &pipe_number, &grade))
    }

    /// Generates a shipping label HTML. Includes pipe specs, customer details,
    /// order number, destination, and other logistics info in a 6-inch wide doc.
    ///
    /// # Errors
    /// - `AppError::PipeNotFound` — pipe ID doesn't exist
    /// - `AppError::Validation` — invalid pipe_type
    pub async fn generate_shipping_label(
        pool: &SqlitePool,
        req: &ShippingLabelRequest,
    ) -> Result<String, AppError> {
        match req.pipe_type.as_str() {
            "seamless" => {
                let pipe = LabelRepo::find_seamless_pipe(pool, req.pipe_id)
                    .await
                    .map_err(AppError::from)?
                    .ok_or_else(|| {
                        AppError::PipeNotFound(format!("Seamless pipe id={}", req.pipe_id))
                    })?;
                Ok(Self::shipping_html(
                    &pipe.pipe_number,
                    &pipe.grade,
                    pipe.od,
                    pipe.wt,
                    pipe.length,
                    pipe.heat_number.as_deref(),
                    pipe.serial_number.as_deref(),
                    &req,
                ))
            }
            "screen" => {
                let pipe = LabelRepo::find_screen_pipe(pool, req.pipe_id)
                    .await
                    .map_err(AppError::from)?
                    .ok_or_else(|| {
                        AppError::PipeNotFound(format!("Screen pipe id={}", req.pipe_id))
                    })?;
                Ok(Self::shipping_html(
                    &pipe.pipe_number,
                    &pipe.base_grade,
                    pipe.base_od,
                    pipe.base_wt,
                    pipe.length,
                    pipe.heat_number.as_deref(),
                    pipe.serial_number.as_deref(),
                    &req,
                ))
            }
            _ => Err(AppError::Validation(format!(
                "Invalid pipe_type '{}'. Must be 'seamless' or 'screen'",
                req.pipe_type
            ))),
        }
    }

    // ━━━ Private HTML generators ━━━

    fn page_style() -> &'static str {
        r#"<style>
  @page { margin: 0.2in; }
  @media print {
    body { margin: 0; padding: 0; }
    .label-page { page-break-after: always; }
  }
  * { box-sizing: border-box; }
</style>"#
    }

    fn seamless_barcode_html(pipe: &crate::models::seamless_pipe::SeamlessPipe) -> String {
        format!(
            r#"<!DOCTYPE html><html><head><meta charset="utf-8">{style}</head><body>
<div class="label-page">
<div style="width:4in;padding:0.15in;font-family:Arial,Helvetica,sans-serif;border:2px solid #000;margin:auto">
  <div style="font-size:22pt;font-weight:bold;text-align:center;letter-spacing:3px;font-family:'Courier New',monospace;border-bottom:2px solid #000;padding-bottom:4px;margin-bottom:6px">{pn}</div>
  <table style="width:100%;font-size:9pt;border-collapse:collapse">
    <tr><td style="padding:2px 4px;font-weight:bold;width:35%">Grade</td><td style="padding:2px 4px">{grade}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">OD x WT</td><td style="padding:2px 4px">{od} x {wt}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Length</td><td style="padding:2px 4px">{len}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Heat #</td><td style="padding:2px 4px">{heat}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Serial #</td><td style="padding:2px 4px">{serial}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Date</td><td style="padding:2px 4px">{date}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Batch</td><td style="padding:2px 4px">{batch}</td></tr>
  </table>
</div>
</div></body></html>"#,
            style = Self::page_style(),
            pn = pipe.pipe_number,
            grade = pipe.grade,
            od = pipe.od,
            wt = pipe.wt,
            len = pipe.length.map(|v| v.to_string()).unwrap_or_default(),
            heat = pipe.heat_number.as_deref().unwrap_or(""),
            serial = pipe.serial_number.as_deref().unwrap_or(""),
            date = pipe.production_date.as_deref().unwrap_or(""),
            batch = pipe.batch_number.as_deref().unwrap_or(""),
        )
    }

    fn screen_barcode_html(pipe: &crate::models::screen_pipe::ScreenPipe) -> String {
        format!(
            r#"<!DOCTYPE html><html><head><meta charset="utf-8">{style}</head><body>
<div class="label-page">
<div style="width:4in;padding:0.15in;font-family:Arial,Helvetica,sans-serif;border:2px solid #000;margin:auto">
  <div style="font-size:22pt;font-weight:bold;text-align:center;letter-spacing:3px;font-family:'Courier New',monospace;border-bottom:2px solid #000;padding-bottom:4px;margin-bottom:6px">{pn}</div>
  <table style="width:100%;font-size:9pt;border-collapse:collapse">
    <tr><td style="padding:2px 4px;font-weight:bold;width:35%">Grade</td><td style="padding:2px 4px">{grade}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Base OD x WT</td><td style="padding:2px 4px">{od} x {wt}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Screen Type</td><td style="padding:2px 4px">{stype}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Slot Size</td><td style="padding:2px 4px">{slot}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Length</td><td style="padding:2px 4px">{len}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Heat #</td><td style="padding:2px 4px">{heat}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Serial #</td><td style="padding:2px 4px">{serial}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Date</td><td style="padding:2px 4px">{date}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Batch</td><td style="padding:2px 4px">{batch}</td></tr>
  </table>
</div>
</div></body></html>"#,
            style = Self::page_style(),
            pn = pipe.pipe_number,
            grade = pipe.base_grade,
            od = pipe.base_od,
            wt = pipe.base_wt,
            stype = pipe.screen_type,
            slot = pipe.slot_size.map(|v| v.to_string()).unwrap_or_default(),
            len = pipe.length.map(|v| v.to_string()).unwrap_or_default(),
            heat = pipe.heat_number.as_deref().unwrap_or(""),
            serial = pipe.serial_number.as_deref().unwrap_or(""),
            date = pipe.production_date.as_deref().unwrap_or(""),
            batch = pipe.batch_number.as_deref().unwrap_or(""),
        )
    }

    fn quality_tag_html(
        cert: &crate::models::quality::QualityCert,
        pipe_number: &str,
        grade: &str,
    ) -> String {
        format!(
            r#"<!DOCTYPE html><html><head><meta charset="utf-8">{style}</head><body>
<div class="label-page">
<div style="width:4in;padding:0.15in;font-family:Arial,Helvetica,sans-serif;border:3px solid #000;margin:auto">
  <div style="font-size:16pt;font-weight:bold;text-align:center;border-bottom:3px double #000;padding-bottom:4px;margin-bottom:8px">QUALITY CERTIFICATE</div>
  <table style="width:100%;font-size:9pt;border-collapse:collapse">
    <tr><td style="padding:2px 4px;font-weight:bold;width:35%">Cert #</td><td style="padding:2px 4px;font-family:'Courier New',monospace;font-weight:bold">{cert}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Pipe #</td><td style="padding:2px 4px">{pn}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Grade</td><td style="padding:2px 4px">{grade}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Result</td><td style="padding:2px 4px;font-weight:bold;color:{color}">{result}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Inspector</td><td style="padding:2px 4px">{inspector}</td></tr>
    <tr><td style="padding:2px 4px;font-weight:bold">Date</td><td style="padding:2px 4px">{date}</td></tr>
  </table>
</div>
</div></body></html>"#,
            style = Self::page_style(),
            cert = cert.cert_number,
            pn = pipe_number,
            grade = grade,
            result = cert.result,
            color = if cert.result.to_lowercase() == "pass" { "#008000" } else { "#cc0000" },
            inspector = cert.inspector.as_deref().unwrap_or(""),
            date = cert.cert_date.as_deref().unwrap_or(""),
        )
    }

    fn shipping_html(
        pipe_number: &str,
        grade: &str,
        od: f64,
        wt: f64,
        length: Option<f64>,
        heat_number: Option<&str>,
        serial_number: Option<&str>,
        req: &ShippingLabelRequest,
    ) -> String {
        format!(
            r#"<!DOCTYPE html><html><head><meta charset="utf-8">{style}</head><body>
<div class="label-page">
<div style="width:6in;padding:0.2in;font-family:Arial,Helvetica,sans-serif;border:2px solid #000;margin:auto">
  <div style="font-size:20pt;font-weight:bold;text-align:center;border-bottom:2px solid #000;padding-bottom:4px;margin-bottom:8px">SHIPPING LABEL</div>
  <div style="display:flex;gap:12px">
    <div style="flex:1">
      <table style="width:100%;font-size:9pt;border-collapse:collapse">
        <tr><td style="padding:2px 4px;font-weight:bold;width:33%">Pipe #</td><td style="padding:2px 4px;font-family:'Courier New',monospace;font-weight:bold">{pn}</td></tr>
        <tr><td style="padding:2px 4px;font-weight:bold">Grade</td><td style="padding:2px 4px">{grade}</td></tr>
        <tr><td style="padding:2px 4px;font-weight:bold">OD x WT</td><td style="padding:2px 4px">{od} x {wt}</td></tr>
        <tr><td style="padding:2px 4px;font-weight:bold">Length</td><td style="padding:2px 4px">{len}</td></tr>
        <tr><td style="padding:2px 4px;font-weight:bold">Heat #</td><td style="padding:2px 4px">{heat}</td></tr>
        <tr><td style="padding:2px 4px;font-weight:bold">Serial #</td><td style="padding:2px 4px">{serial}</td></tr>
      </table>
    </div>
    <div style="flex:1;border-left:1px solid #999;padding-left:8px">
      <table style="width:100%;font-size:9pt;border-collapse:collapse">
        <tr><td style="padding:2px 4px;font-weight:bold">Order #</td><td style="padding:2px 4px">{order}</td></tr>
        <tr><td style="padding:2px 4px;font-weight:bold">Customer</td><td style="padding:2px 4px">{customer}</td></tr>
        <tr><td style="padding:2px 4px;font-weight:bold">PO #</td><td style="padding:2px 4px">{po}</td></tr>
        <tr><td style="padding:2px 4px;font-weight:bold">Destination</td><td style="padding:2px 4px">{dest}</td></tr>
        <tr><td style="padding:2px 4px;font-weight:bold">Ship Date</td><td style="padding:2px 4px">{ship}</td></tr>
      </table>
    </div>
  </div>
</div>
</div></body></html>"#,
            style = Self::page_style(),
            pn = pipe_number,
            grade = grade,
            od = od,
            wt = wt,
            len = length.map(|v| v.to_string()).unwrap_or_default(),
            heat = heat_number.unwrap_or(""),
            serial = serial_number.unwrap_or(""),
            order = req.order_number.as_deref().unwrap_or(""),
            customer = req.customer_name.as_deref().unwrap_or(""),
            po = req.po_number.as_deref().unwrap_or(""),
            dest = req.destination.as_deref().unwrap_or(""),
            ship = req.ship_date.as_deref().unwrap_or(""),
        )
    }

    fn batch_html(labels: &[String]) -> String {
        let body: String = labels.iter().map(|l| {
            let inner = l
                .lines()
                .skip_while(|line| !line.contains("<div class=\"label-page\""))
                .collect::<Vec<_>>()
                .join("\n");
            inner
        }).collect::<Vec<_>>().join("\n");

        format!(
            r#"<!DOCTYPE html><html><head><meta charset="utf-8">{style}</head><body>
{body}
</body></html>"#,
            style = Self::page_style(),
            body = body,
        )
    }
}
