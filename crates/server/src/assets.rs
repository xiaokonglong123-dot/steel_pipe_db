//! 嵌入式前端资源（`web/dist`）。单二进制部署：无独立静态目录。

use axum::body::Body;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../web/dist"]
struct WebAssets;

/// SPA 兜底：命中静态文件则直接返回，否则回退到 index.html（客户端路由）。
pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    if path.starts_with("api/") {
        return StatusCode::NOT_FOUND.into_response();
    }

    if let Some(response) = serve_file(path) {
        return response;
    }

    serve_file("index.html").unwrap_or_else(|| {
        (
            StatusCode::OK,
            "前端尚未构建：cd web && npm ci && npm run build",
        )
            .into_response()
    })
}

fn serve_file(path: &str) -> Option<Response> {
    let file = WebAssets::get(path)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let cache = if path == "index.html" {
        "no-cache"
    } else {
        // Vite 产物文件名带内容哈希
        "public, max-age=31536000, immutable"
    };

    let mut response = Response::new(Body::from(file.data.into_owned()));
    let headers = response.headers_mut();
    headers.insert(CONTENT_TYPE, mime.as_ref().parse().ok()?);
    headers.insert(CACHE_CONTROL, cache.parse().ok()?);
    Some(response)
}
