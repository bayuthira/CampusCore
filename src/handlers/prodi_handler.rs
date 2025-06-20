// src/handlers/prodi_handler.rs
use crate::{
    db::DbPool,
    errors::AppError,
    models::prodi_model::{CreateProdiPayload, Prodi},
    repositories::prodi_repo,
};
use axum::{extract::State, http::StatusCode, Json};

// Handler untuk membuat prodi baru
pub async fn create_prodi_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateProdiPayload>, // Axum otomatis mendeserialisasi body request
) -> Result<(StatusCode, Json<Prodi>), AppError> {
    let prodi = prodi_repo::create_prodi_repo(&pool, payload).await?;

    Ok((StatusCode::CREATED, Json(prodi)))
}

// Handler untuk mendapatkan semua prodi
pub async fn get_all_prodi_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<Prodi>>, AppError> {
    let prodi_list = prodi_repo::get_all_prodi_repo(&pool).await?;

    Ok(Json(prodi_list))
}