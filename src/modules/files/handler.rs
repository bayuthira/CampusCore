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
    Path((folder, filename)): Path<(String, String)>,
) -> Result<Response<Body>, AppError> {
    let path_str = format!("uploads/{}/{}", folder, filename);
    let path = StdPath::new(&path_str);

    if !path.starts_with("uploads/") || !path.exists() || !path.is_file() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let file_contents = tokio::fs::read(path).await?;
    let mime_type = mime_guess::from_path(path).first_or_octet_stream().to_string();

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .body(Body::from(file_contents))
        .unwrap();

    Ok(response)
}