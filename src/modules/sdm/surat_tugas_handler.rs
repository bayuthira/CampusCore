// src/modules/sdm/surat_tugas_handler.rs

use super::{
    surat_tugas_model::{
        CreateSuratTugasPayload, SuratTugas, SuratTugasDetail, UpdateSuratTugasPayload,
    },
    surat_tugas_repo as repo,
};
use crate::{modules::auth::middleware::TokenClaims, db::DbPool, errors::AppError};
use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    Extension,
};
use uuid::Uuid;

/// Handler untuk membuat Surat Tugas baru
pub async fn create_surat_tugas_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<CreateSuratTugasPayload>,
) -> Result<(StatusCode, Json<SuratTugasDetail>), AppError> {
    let user_pembuat_id = claims.sub;
    let surat_tugas =
        repo::create_surat_tugas_repo(&pool, user_pembuat_id, payload).await?;
    Ok((StatusCode::CREATED, Json(surat_tugas)))
}

/// Handler untuk mendapatkan daftar semua Surat Tugas (versi ringan)
pub async fn get_all_surat_tugas_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<SuratTugas>>, AppError> {
    let list = repo::get_all_surat_tugas_repo(&pool).await?;
    Ok(Json(list))
}

/// Handler untuk mendapatkan detail satu Surat Tugas (versi lengkap)
pub async fn get_surat_tugas_detail_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<SuratTugasDetail>, AppError> {
    let detail = repo::get_surat_tugas_detail_repo(&pool, id).await?;
    Ok(Json(detail))
}

/// Handler untuk memperbarui Surat Tugas
pub async fn update_surat_tugas_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSuratTugasPayload>,
) -> Result<Json<SuratTugasDetail>, AppError> {
    let updated_surat_tugas = repo::update_surat_tugas_repo(&pool, id, payload).await?;
    Ok(Json(updated_surat_tugas))
}

/// Handler untuk menghapus Surat Tugas
pub async fn delete_surat_tugas_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    repo::delete_surat_tugas_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}