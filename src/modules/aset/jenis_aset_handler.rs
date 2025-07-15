// src/handlers/jenis_aset_handler.rs
use crate::{db::DbPool, errors::AppError, models::jenis_aset_model::{JenisAset, JenisAsetPayload}, repositories::jenis_aset_repo};
use axum::{extract::{Path, State, Json}, http::StatusCode};
use uuid::Uuid;

pub async fn create_jenis_aset_handler(State(pool): State<DbPool>, Json(payload): Json<JenisAsetPayload>) -> Result<(StatusCode, Json<JenisAset>), AppError> {
    let jenis_aset = jenis_aset_repo::create_jenis_aset_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(jenis_aset)))
}

pub async fn get_all_jenis_aset_handler(State(pool): State<DbPool>) -> Result<Json<Vec<JenisAset>>, AppError> {
    let list = jenis_aset_repo::get_all_jenis_aset_repo(&pool).await?;
    Ok(Json(list))
}

pub async fn get_jenis_aset_by_id_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> Result<Json<JenisAset>, AppError> {
    let jenis_aset = jenis_aset_repo::get_jenis_aset_by_id_repo(&pool, id).await?;
    Ok(Json(jenis_aset))
}

pub async fn update_jenis_aset_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>, Json(payload): Json<JenisAsetPayload>) -> Result<Json<JenisAset>, AppError> {
    let updated = jenis_aset_repo::update_jenis_aset_repo(&pool, id, payload).await?;
    Ok(Json(updated))
}

pub async fn delete_jenis_aset_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    jenis_aset_repo::delete_jenis_aset_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}