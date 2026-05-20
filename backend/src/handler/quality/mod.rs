use std::sync::Arc;
use std::path::Path;

use axum::{
    extract::{Multipart, Path as AxumPath, Query, State},
    Json,
};
use uuid::Uuid;

use crate::domain::quality::{CreateQualityCertDto, QualityCertFilter, TraceResult, UpdateQualityCertDto};
use crate::domain::{Api5ctGradeRef, PipeAttachment, QualityCert};
use crate::error::AppResult;
use crate::middleware::AuthUser;
use crate::service::quality_service::QualityService;
use crate::AppState;

use super::{list_response, ok_response};

pub async fn list_certs(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
    Query(filter): Query<QualityCertFilter>,
) -> AppResult<Json<super::ApiListResponse<QualityCert>>> {
    let (certs, total) = QualityService::list_certs(&state.db, filter).await?;
    let page = 1i64;
    let page_size = 20i64;
    Ok(list_response(certs, total, page, page_size))
}

pub async fn create_cert(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
    Json(dto): Json<CreateQualityCertDto>,
) -> AppResult<Json<super::ApiResponse<QualityCert>>> {
    let cert = QualityService::create_cert(&state.db, dto).await?;
    Ok(ok_response(cert))
}

pub async fn get_cert(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
    AxumPath(id): AxumPath<String>,
) -> AppResult<Json<super::ApiResponse<QualityCert>>> {
    let cert = QualityService::get_cert(&state.db, &id).await?;
    Ok(ok_response(cert))
}

pub async fn update_cert(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
    AxumPath(id): AxumPath<String>,
    Json(dto): Json<UpdateQualityCertDto>,
) -> AppResult<Json<super::ApiResponse<QualityCert>>> {
    let cert = QualityService::update_cert(&state.db, &id, dto).await?;
    Ok(ok_response(cert))
}

pub async fn trace_by_heat_number(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
    AxumPath(heat_no): AxumPath<String>,
) -> AppResult<Json<super::ApiResponse<Vec<TraceResult>>>> {
    let results = QualityService::trace_by_heat_number(&state.db, &heat_no).await?;
    Ok(ok_response(results))
}

pub async fn trace_by_pipe_number(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
    AxumPath(pipe_no): AxumPath<String>,
) -> AppResult<Json<super::ApiResponse<Vec<TraceResult>>>> {
    let results = QualityService::trace_by_pipe_number(&state.db, &pipe_no).await?;
    Ok(ok_response(results))
}

pub async fn list_grades(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
) -> AppResult<Json<super::ApiResponse<Vec<Api5ctGradeRef>>>> {
    let grades = QualityService::list_api_5ct_grades(&state.db).await?;
    Ok(ok_response(grades))
}

pub async fn get_grade(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
    AxumPath(grade): AxumPath<String>,
) -> AppResult<Json<super::ApiResponse<Api5ctGradeRef>>> {
    let grade_ref = QualityService::get_api_5ct_reference(&state.db, &grade).await?;
    Ok(ok_response(grade_ref))
}

pub async fn upload_attachment(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    mut multipart: Multipart,
) -> AppResult<Json<super::ApiResponse<PipeAttachment>>> {
    let mut pipe_type = String::new();
    let mut pipe_id = String::new();
    let mut file_data: Option<(String, Vec<u8>, String)> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "pipe_type" => pipe_type = field.text().await.unwrap_or_default(),
            "pipe_id" => pipe_id = field.text().await.unwrap_or_default(),
            "file" => {
                let file_name = field
                    .file_name()
                    .unwrap_or("unnamed")
                    .to_string();
                let mime_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let bytes = field.bytes().await.unwrap_or_default().to_vec();
                file_data = Some((file_name, bytes, mime_type));
            }
            _ => {}
        }
    }

    if pipe_type.is_empty() || pipe_id.is_empty() {
        return Err(crate::error::AppError::BadRequest("pipe_type and pipe_id are required".into()));
    }

    let (file_name, bytes, mime_type) = file_data.ok_or_else(|| {
        crate::error::AppError::BadRequest("file field is required".into())
    })?;

    let upload_dir = &state.config.upload_dir;
    tokio::fs::create_dir_all(upload_dir).await?;

    let ext = Path::new(&file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let stored_name = format!("{}_{}.{}", Uuid::new_v4().to_string().replace("-", ""), &file_name, ext);
    let file_path = upload_dir.join(&stored_name);

    tokio::fs::write(&file_path, &bytes).await?;

    let file_size = bytes.len() as i64;

    let attachment = QualityService::attach_file(
        &state.db,
        &pipe_type,
        &pipe_id,
        &file_name,
        &file_path.to_string_lossy(),
        file_size,
        &mime_type,
        &auth_user.user_id,
    )
    .await?;

    Ok(ok_response(attachment))
}

pub async fn delete_attachment(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
    AxumPath(id): AxumPath<String>,
) -> AppResult<Json<super::ApiResponse<PipeAttachment>>> {
    let attachment = QualityService::delete_attachment(&state.db, &id).await?;
    Ok(ok_response(attachment))
}
