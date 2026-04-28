// src/modules/matakuliah/handler.rs

use super::{
    model::{
        CreateMataKuliahPayload, MataKuliahDetail, UpdateMataKuliahPayload, VerifikasiRpsPayload,
    },
    repo as matakuliah_repo,
};

use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};

use axum::{
    Extension,
    extract::{Json, Path, State},
    http::StatusCode,
};
use uuid::Uuid;

/// Handler untuk membuat Mata Kuliah baru
pub async fn create_matakuliah_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateMataKuliahPayload>,
) -> Result<(StatusCode, Json<MataKuliahDetail>), AppError> {
    let new_mk = matakuliah_repo::create_matakuliah_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(new_mk)))
}

/// Handler untuk mendapatkan semua Mata Kuliah
pub async fn get_all_matakuliah_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<MataKuliahDetail>>, AppError> {
    let mk_list = matakuliah_repo::get_all_matakuliah_repo(&pool).await?;
    Ok(Json(mk_list))
}

/// Handler untuk mendapatkan satu Mata Kuliah berdasarkan ID
pub async fn get_matakuliah_by_id_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<MataKuliahDetail>, AppError> {
    let mk = matakuliah_repo::get_matakuliah_by_id_repo(&pool, id).await?;
    Ok(Json(mk))
}

/// Handler untuk memperbarui Mata Kuliah
pub async fn update_matakuliah_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<TokenClaims>, // <-- Ambil info user
    Json(payload): Json<UpdateMataKuliahPayload>,
) -> Result<Json<MataKuliahDetail>, AppError> {
    // Cek apakah ada upaya untuk mengubah Kode MK
    if let Some(ref _kode_mk) = payload.kode_mk {
        // Jika ada, hanya SUPER_ADMIN yang boleh melanjutkan
        if !claims.roles.contains(&"SUPER_ADMIN".to_string()) {
            return Err(AppError::Forbidden(
                "Hanya SUPER_ADMIN yang dapat mengubah Kode MK.".to_string(),
            ));
        }
    }

    let updated_mk = matakuliah_repo::update_matakuliah_repo(&pool, id, payload).await?;
    Ok(Json(updated_mk))
}

/// Handler untuk menghapus Mata Kuliah
pub async fn delete_matakuliah_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    matakuliah_repo::delete_matakuliah_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn verifikasi_rps_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<VerifikasiRpsPayload>,
) -> Result<Json<MataKuliahDetail>, AppError> {
    let updated_mk = matakuliah_repo::verifikasi_rps_repo(&pool, id, payload).await?;
    Ok(Json(updated_mk))
}
