// src/handlers/dosen_handler.rs
use super::{
    model::{CreateDosenPayload, DosenDetail, UpdateDosenPayload},
    repo as dosen_repo, // Gunakan alias agar panggilan fungsi tetap sama
};
use crate::{db::DbPool, errors::AppError};
use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    Extension,
};
use uuid::Uuid;

/// Handler untuk membuat Dosen baru.
/// Axum akan secara otomatis mengekstrak State (DbPool) dan body JSON (payload).
pub async fn create_dosen_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateDosenPayload>,
) -> Result<(StatusCode, Json<DosenDetail>), AppError> {
    // Memanggil fungsi repository untuk menyimpan data ke database.
    // Operator `?` akan otomatis mengubah error dari repository menjadi AppError.
    let created_dosen = dosen_repo::create_dosen_repo(&pool, payload).await?;

    // Jika berhasil, kembalikan status 201 Created bersama dengan data dosen yang baru dibuat.
    Ok((StatusCode::CREATED, Json(created_dosen)))
}

/// Handler untuk mendapatkan semua data Dosen.
pub async fn get_all_dosen_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<DosenDetail>>, AppError> {
    // Memanggil fungsi repository untuk mengambil semua data dari database.
    let dosen_list = dosen_repo::get_all_dosen_repo(&pool).await?;

    // Jika berhasil, kembalikan status 200 OK dengan daftar dosen dalam format JSON.
    Ok(Json(dosen_list))
}

/// Handler untuk mendapatkan detail satu dosen berdasarkan ID
pub async fn get_dosen_by_id_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>, // Axum akan mengekstrak ID dari URL path
) -> Result<Json<DosenDetail>, AppError> {
    let dosen = dosen_repo::get_dosen_by_id_repo(&pool, id).await?;

    Ok(Json(dosen))
}

pub async fn update_dosen_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateDosenPayload>,
) -> Result<Json<DosenDetail>, AppError> {
    let updated_dosen = dosen_repo::update_dosen_repo(&pool, id, payload).await?;
    Ok(Json(updated_dosen))
}

pub async fn delete_dosen_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    dosen_repo::delete_dosen_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}