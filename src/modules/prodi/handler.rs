// src/modules/prodi/handler.rs
use super::{
    model::{CreateProdiPayload, Prodi, UpdateProdiPayload},
    repo as prodi_repo,
};
use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};
use axum::{
    Extension,
    extract::{Json, Path, State},
    http::StatusCode,
};
use uuid::Uuid;

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

pub async fn get_prodi_by_id_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Prodi>, AppError> {
    let prodi = prodi_repo::get_prodi_by_id_repo(&pool, id).await?;
    Ok(Json(prodi))
}

pub async fn update_prodi_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<UpdateProdiPayload>,
) -> Result<Json<Prodi>, AppError> {
    if payload.kode_prodi.is_some() {
        if !claims.roles.contains(&"SUPER_ADMIN".to_string()) {
            return Err(AppError::Forbidden(
                "Hanya SUPER_ADMIN yang dapat mengubah Kode Prodi.".to_string(),
            ));
        }
    }
    let updated_prodi = prodi_repo::update_prodi_repo(&pool, id, payload).await?;
    Ok(Json(updated_prodi))
}

pub async fn delete_prodi_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    prodi_repo::delete_prodi_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
