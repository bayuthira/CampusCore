// src/modules/files/handler.rs
use crate::errors::AppError;
use axum::{
    body::Body,
    extract::Path,
    http::{header, Response, StatusCode},
    response::IntoResponse,
};
use std::path::Path as StdPath;

pub async fn serve_file_handler(
    Path(path): Path<String>,
) -> Result<Response<Body>, AppError> {
    
    // Gabungkan "uploads" dengan path yang diterima dari URL
    // `path` akan berisi "sdm/64f6832b.../105c4543....png"
    let path_str = format!("uploads/{}", path);
    let path = StdPath::new(&path_str);

    // Validasi keamanan (Path Traversal Prevention)
    // Pastikan path yang diminta benar-benar ada di dalam folder 'uploads'
    let canonical_uploads = tokio::fs::canonicalize("./uploads").await?;
    let canonical_path = tokio::fs::canonicalize(path).await?;

    if !canonical_path.starts_with(canonical_uploads) {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    if !path.exists() || !path.is_file() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    // Baca file dan kirim sebagai respons
    let file_contents = tokio::fs::read(path).await?;
    let mime_type = mime_guess::from_path(path).first_or_octet_stream().to_string();

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .body(Body::from(file_contents))
        .unwrap();

    Ok(response)
}