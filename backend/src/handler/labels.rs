use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::header,
    response::IntoResponse,
    Extension, Json,
};

use crate::domain::labels::{
    GenerateLabelsRequest, LabelTemplateDto, UpdateLabelTemplateDto,
};
use crate::error::AppResult;
use crate::handler::{list_response, ok_response};
use crate::middleware::AuthUser;
use crate::service::label_service::TemplateListFilter;
use crate::AppState;

pub async fn list_templates(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<TemplateListFilter>,
) -> AppResult<Json<impl serde::Serialize>> {
    let (templates, total) = state.label_service.list_templates(&filter).await?;
    Ok(list_response(templates, total, filter.page, filter.page_size))
}

pub async fn create_template(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Json(dto): Json<LabelTemplateDto>,
) -> AppResult<Json<impl serde::Serialize>> {
    let template = state.label_service.create_template(dto).await?;
    Ok(ok_response(template))
}

pub async fn update_template(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateLabelTemplateDto>,
) -> AppResult<Json<impl serde::Serialize>> {
    let template = state.label_service.update_template(&id, dto).await?;
    Ok(ok_response(template))
}

pub async fn delete_template(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> AppResult<Json<impl serde::Serialize>> {
    state.label_service.delete_template(&id).await?;
    Ok(ok_response(serde_json::json!({"deleted": true, "id": id})))
}

pub async fn generate_labels(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthUser>,
    Json(dto): Json<GenerateLabelsRequest>,
) -> AppResult<impl IntoResponse> {
    let pdf_bytes = state
        .label_service
        .generate_labels_bytes(dto, &auth.username)
        .await?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"labels.pdf\"",
        ),
    ];

    Ok((axum::http::StatusCode::OK, headers, pdf_bytes))
}

pub async fn print_history(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<impl serde::Serialize>> {
    let (logs, total) = state.label_service.print_history(1, 50).await?;
    Ok(list_response(logs, total, 1, 50))
}
