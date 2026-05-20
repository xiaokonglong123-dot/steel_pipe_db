use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};

use crate::error::AppResult;
use crate::handler::{list_response, ok_response};
use crate::middleware::AuthUser;
use crate::service::pipe_service::{CreateScreenPipeDto, CreateSeamlessPipeDto, PipeListFilter, PipeService};
use crate::service::pipe_service::{UpdateScreenPipeDto, UpdateSeamlessPipeDto};
use crate::AppState;

// ── Seamless pipe handlers ───────────────────────────────────────────────────

pub async fn list_seamless_pipes(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<PipeListFilter>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = PipeService::new(state.db.clone(), state.config.clone());
    let (pipes, total) = service.list_seamless_pipes(&filter).await?;
    Ok(list_response(pipes, total, filter.page, filter.page_size))
}

pub async fn create_seamless_pipe(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Json(dto): Json<CreateSeamlessPipeDto>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = PipeService::new(state.db.clone(), state.config.clone());
    let pipe = service.create_seamless_pipe(dto).await?;
    Ok(ok_response(pipe))
}

pub async fn get_seamless_pipe(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = PipeService::new(state.db.clone(), state.config.clone());
    let pipe = service.get_seamless_pipe(&id).await?;
    Ok(ok_response(pipe))
}

pub async fn update_seamless_pipe(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateSeamlessPipeDto>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = PipeService::new(state.db.clone(), state.config.clone());
    let pipe = service.update_seamless_pipe(&id, dto).await?;
    Ok(ok_response(pipe))
}

pub async fn delete_seamless_pipe(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = PipeService::new(state.db.clone(), state.config.clone());
    service.delete_seamless_pipe(&id).await?;
    Ok(ok_response(serde_json::json!({"deleted": true, "id": id})))
}

// ── Screen pipe handlers ────────────────────────────────────────────────────

pub async fn list_screen_pipes(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<PipeListFilter>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = PipeService::new(state.db.clone(), state.config.clone());
    let (pipes, total) = service.list_screen_pipes(&filter).await?;
    Ok(list_response(pipes, total, filter.page, filter.page_size))
}

pub async fn create_screen_pipe(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Json(dto): Json<CreateScreenPipeDto>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = PipeService::new(state.db.clone(), state.config.clone());
    let pipe = service.create_screen_pipe(dto).await?;
    Ok(ok_response(pipe))
}

pub async fn get_screen_pipe(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = PipeService::new(state.db.clone(), state.config.clone());
    let pipe = service.get_screen_pipe(&id).await?;
    Ok(ok_response(pipe))
}

pub async fn update_screen_pipe(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateScreenPipeDto>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = PipeService::new(state.db.clone(), state.config.clone());
    let pipe = service.update_screen_pipe(&id, dto).await?;
    Ok(ok_response(pipe))
}

pub async fn delete_screen_pipe(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = PipeService::new(state.db.clone(), state.config.clone());
    service.delete_screen_pipe(&id).await?;
    Ok(ok_response(serde_json::json!({"deleted": true, "id": id})))
}

// ── Tracing handlers ─────────────────────────────────────────────────────────

pub async fn trace_by_pipe_number(
    State(state): State<Arc<AppState>>,
    Path(pipe_no): Path<String>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = PipeService::new(state.db.clone(), state.config.clone());
    let result = service.trace_by_pipe_number(&pipe_no).await?;
    Ok(ok_response(result))
}

pub async fn trace_by_heat_number(
    State(state): State<Arc<AppState>>,
    Path(heat_no): Path<String>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = PipeService::new(state.db.clone(), state.config.clone());
    let results = service.trace_by_heat_number(&heat_no).await?;
    Ok(ok_response(results))
}
