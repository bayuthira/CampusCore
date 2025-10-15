// src/modules/sdm/handler.rs

use super::{
    model::{Pegawai, PegawaiPayload, CreateUserForPegawaiPayload},
    repo, // Mengacu pada src/modules/sdm/repo.rs
};
use crate::{db::DbPool, errors::AppError};
use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
};
use uuid::Uuid;

/// Handler untuk membuat Pegawai baru (dan User-nya)
pub async fn create_pegawai_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<PegawaiPayload>,
) -> Result<(StatusCode, Json<Pegawai>), AppError> {
    let pegawai = repo::create_pegawai_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(pegawai)))
}

/// Handler untuk mendapatkan semua Pegawai
pub async fn get_all_pegawai_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<Pegawai>>, AppError> {
    let pegawai_list = repo::get_all_pegawai_repo(&pool).await?;
    Ok(Json(pegawai_list))
}

/// Handler untuk mendapatkan satu Pegawai berdasarkan ID
pub async fn get_pegawai_by_id_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Pegawai>, AppError> {
    let pegawai = repo::get_pegawai_by_id_repo(&pool, id).await?;
    Ok(Json(pegawai))
}

/// Handler untuk memperbarui data Pegawai
pub async fn update_pegawai_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<PegawaiPayload>,
) -> Result<Json<Pegawai>, AppError> {
    let updated_pegawai = repo::update_pegawai_repo(&pool, id, payload).await?;
    Ok(Json(updated_pegawai))
}

/// Handler untuk menghapus Pegawai
pub async fn delete_pegawai_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    repo::delete_pegawai_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}


pub async fn create_user_for_pegawai_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateUserForPegawaiPayload>,
) -> Result<Json<Pegawai>, AppError> {
    let updated_pegawai = repo::create_user_for_pegawai_repo(&pool, id, payload).await?;
    Ok(Json(updated_pegawai))
}