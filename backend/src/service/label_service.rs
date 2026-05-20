use std::io::BufWriter;

use printpdf::*;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::domain::common::LabelTemplate;
use crate::domain::labels::{
    GenerateLabelsRequest, LabelGenerateResult, LabelTemplateDto, PrintLog,
    UpdateLabelTemplateDto,
};
use crate::error::{AppError, AppResult};
use crate::repository::label_repo::{LabelTemplateRepo, PrintLogRepo};

#[derive(Debug, Deserialize)]
pub struct TemplateListFilter {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    #[serde(default = "default_sort_by")]
    pub sort_by: String,
    #[serde(default = "default_sort_order")]
    pub sort_order: String,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 20 }
fn default_sort_by() -> String { "created_at".to_string() }
fn default_sort_order() -> String { "desc".to_string() }

#[derive(Serialize)]
struct PipeLabelData {
    pipe_number: String,
    grade: String,
    spec: String,
    length: f64,
    weight: f64,
    heat_number: String,
    production_date: String,
    qr_url: String,
}

#[derive(Debug, sqlx::FromRow)]
struct SeamlessPipeRow {
    #[allow(dead_code)]
    id: String,
    pipe_number: String,
    grade: String,
    od: f64,
    wt: f64,
    length: f64,
    weight: f64,
    heat_number: Option<String>,
    production_date: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct ScreenPipeRow {
    #[allow(dead_code)]
    id: String,
    pipe_number: String,
    grade: String,
    od: f64,
    wt: f64,
    length: f64,
    weight: f64,
    heat_number: Option<String>,
    production_date: Option<String>,
}

pub struct LabelService {
    pool: SqlitePool,
}

impl LabelService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    fn template_repo(&self) -> LabelTemplateRepo {
        LabelTemplateRepo::new(self.pool.clone())
    }

    fn print_log_repo(&self) -> PrintLogRepo {
        PrintLogRepo::new(self.pool.clone())
    }

    pub async fn create_template(&self, dto: LabelTemplateDto) -> AppResult<LabelTemplate> {
        self.template_repo().create(&dto).await
    }

    pub async fn update_template(
        &self,
        id: &str,
        dto: UpdateLabelTemplateDto,
    ) -> AppResult<LabelTemplate> {
        self.template_repo().update(id, &dto).await
    }

    pub async fn delete_template(&self, id: &str) -> AppResult<()> {
        self.template_repo().delete(id).await
    }

    pub async fn list_templates(
        &self,
        _filter: &TemplateListFilter,
    ) -> AppResult<(Vec<LabelTemplate>, i64)> {
        let templates = self.template_repo().list().await?;
        let total = templates.len() as i64;
        Ok((templates, total))
    }

    pub async fn get_template(&self, id: &str) -> AppResult<LabelTemplate> {
        self.template_repo().find_by_id(id).await
    }

    pub async fn generate_labels_bytes(
        &self,
        req: GenerateLabelsRequest,
        printed_by: &str,
    ) -> AppResult<Vec<u8>> {
        let template = self.template_repo().find_by_id(&req.template_id).await?;
        if req.pipe_numbers.is_empty() {
            return Err(AppError::BadRequest("pipe_numbers must not be empty".into()));
        }
        let pipe_details = self.fetch_pipe_details(&req.pipe_numbers).await?;
        let pdf_bytes = self.build_label_pdf(&template, &pipe_details)?;
        self.print_log_repo()
            .insert(&template.id, &template.name, &req.pipe_numbers,
                    req.pipe_numbers.len() as i64, printed_by)
            .await?;
        Ok(pdf_bytes)
    }

    pub async fn generate_labels(
        &self,
        req: GenerateLabelsRequest,
        printed_by: &str,
    ) -> AppResult<LabelGenerateResult> {
        let template = self.template_repo().find_by_id(&req.template_id).await?;
        if req.pipe_numbers.is_empty() {
            return Err(AppError::BadRequest("pipe_numbers must not be empty".into()));
        }
        let pipe_details = self.fetch_pipe_details(&req.pipe_numbers).await?;
        let pdf_bytes = self.build_label_pdf(&template, &pipe_details)?;
        self.print_log_repo()
            .insert(&template.id, &template.name, &req.pipe_numbers,
                    req.pipe_numbers.len() as i64, printed_by)
            .await?;
        Ok(LabelGenerateResult {
            template_id: template.id,
            template_name: template.name,
            total_labels: req.pipe_numbers.len(),
            pdf_size_bytes: pdf_bytes.len(),
        })
    }

    pub async fn print_history(
        &self,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<PrintLog>, i64)> {
        self.print_log_repo().list(page, page_size).await
    }

    async fn fetch_pipe_details(&self, pipe_numbers: &[String]) -> AppResult<Vec<PipeLabelData>> {
        let mut result = Vec::with_capacity(pipe_numbers.len());
        for pn in pipe_numbers {
            result.push(self.fetch_single_pipe(pn).await?);
        }
        Ok(result)
    }

    async fn fetch_single_pipe(&self, pipe_number: &str) -> AppResult<PipeLabelData> {
        let row = sqlx::query_as::<_, SeamlessPipeRow>(
            "SELECT id, pipe_number, grade, od, wt, length, weight, \
             heat_number, production_date FROM seamless_pipes \
             WHERE pipe_number = ?1 AND deleted_at IS NULL",
        )
        .bind(pipe_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        if let Some(r) = row {
            let pipe_number = r.pipe_number;
            return Ok(PipeLabelData {
                spec: format!("{:.3}in x {:.2}lb", r.od, r.wt),
                pipe_number: pipe_number.clone(),
                grade: r.grade,
                length: r.length,
                weight: r.weight,
                heat_number: r.heat_number.unwrap_or_default(),
                production_date: r.production_date.unwrap_or_default(),
                qr_url: format!(
                    "http://localhost:8080/api/v1/trace/pipe-number/{}", pipe_number
                ),
            });
        }

        let row = sqlx::query_as::<_, ScreenPipeRow>(
            "SELECT id, pipe_number, grade, od, wt, length, weight, \
             heat_number, production_date FROM screen_pipes \
             WHERE pipe_number = ?1 AND deleted_at IS NULL",
        )
        .bind(pipe_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        if let Some(r) = row {
            let pipe_number = r.pipe_number;
            return Ok(PipeLabelData {
                spec: format!("{:.3}in x {:.2}lb", r.od, r.wt),
                pipe_number: pipe_number.clone(),
                grade: r.grade,
                length: r.length,
                weight: r.weight,
                heat_number: r.heat_number.unwrap_or_default(),
                production_date: r.production_date.unwrap_or_default(),
                qr_url: format!(
                    "http://localhost:8080/api/v1/trace/pipe-number/{}", pipe_number
                ),
            });
        }

        Err(AppError::NotFound(format!(
            "Pipe with number '{}' not found", pipe_number
        )))
    }

    fn build_label_pdf(
        &self,
        template: &LabelTemplate,
        pipes: &[PipeLabelData],
    ) -> AppResult<Vec<u8>> {
        let label_w_mm = template.width_mm.clamp(40.0, 100.0);
        let label_h_mm = template.height_mm.clamp(30.0, 80.0);
        let margin_mm = 10.0;
        let gap_mm = 5.0;

        let cols = ((210.0 - 2.0 * margin_mm + gap_mm) / (label_w_mm + gap_mm)).floor() as usize;
        let cols = cols.max(1);

        let rows_per_page = ((297.0 - 2.0 * margin_mm + gap_mm) / (label_h_mm + gap_mm)).floor()
            as usize;
        let rows_per_page = rows_per_page.max(1);
        let labels_per_page = cols * rows_per_page;

        let (doc, page1, layer1) =
            PdfDocument::new("Labels", Mm(210.0), Mm(297.0), "Layer 1");

        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| AppError::Internal(format!("Font error: {}", e)))?;
        let font_bold = doc
            .add_builtin_font(BuiltinFont::HelveticaBold)
            .map_err(|e| AppError::Internal(format!("Font error: {}", e)))?;

        let mut first_page = true;

        for chunk in pipes.chunks(labels_per_page) {
            let layer = if first_page {
                first_page = false;
                doc.get_page(page1).get_layer(layer1)
            } else {
                let (page_idx, layer_idx) = doc.add_page(Mm(210.0), Mm(297.0), "Labels");
                doc.get_page(page_idx).get_layer(layer_idx)
            };

            for (i, pipe) in chunk.iter().enumerate() {
                let col = i % cols;
                let row = i / cols;
                let x = (margin_mm + col as f64 * (label_w_mm + gap_mm)) as f32;
                let y = (margin_mm + row as f64 * (label_h_mm + gap_mm)) as f32;

                self.draw_label(
                    &layer,
                    Mm(x),
                    Mm(y),
                    label_w_mm as f32,
                    label_h_mm as f32,
                    &font_bold,
                    &font,
                    pipe,
                );
            }
        }

        let mut buf = BufWriter::new(Vec::new());
        doc.save(&mut buf)
            .map_err(|e| AppError::Internal(format!("PDF save error: {}", e)))?;
        Ok(buf.into_inner().unwrap())
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_label(
        &self,
        layer: &PdfLayerReference,
        x: Mm,
        y: Mm,
        w_mm: f32,
        h_mm: f32,
        font_bold: &IndirectFontRef,
        font: &IndirectFontRef,
        pipe: &PipeLabelData,
    ) {
        let rect = Rect::new(Mm(x.0), Mm(y.0), Mm(x.0 + w_mm), Mm(y.0 + h_mm));
        layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
        layer.set_outline_thickness(0.5);
        layer.add_rect(rect);

        let text_x = Mm(x.0 + 2.0);
        let mut text_y = Mm(y.0 + h_mm - 4.0);
        let line_h = 3.5f32;

        layer.use_text(pipe.pipe_number.clone(), 8.0, text_x, text_y, font_bold);
        text_y.0 -= line_h + 1.0;

        layer.use_text(
            format!("Grade: {}", pipe.grade),
            6.0,
            text_x,
            text_y,
            font,
        );
        text_y.0 -= line_h;

        layer.use_text(
            format!("Spec: {}", pipe.spec),
            6.0,
            text_x,
            text_y,
            font,
        );
        text_y.0 -= line_h;

        layer.use_text(
            format!("L: {:.2}m  W: {:.2}kg", pipe.length, pipe.weight),
            6.0,
            text_x,
            text_y,
            font,
        );
        text_y.0 -= line_h;

        if !pipe.heat_number.is_empty() {
            layer.use_text(
                format!("Heat: {}", pipe.heat_number),
                6.0,
                text_x,
                text_y,
                font,
            );
            text_y.0 -= line_h;
        }

        if !pipe.production_date.is_empty() {
            layer.use_text(
                format!("Date: {}", pipe.production_date),
                6.0,
                text_x,
                text_y,
                font,
            );
        }
    }
}
