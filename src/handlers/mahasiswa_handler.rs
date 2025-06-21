// src/handlers/mahasiswa_handler.rs

use crate::{
    db::DbPool,
    errors::AppError,
    models::mahasiswa_model::{CreateMahasiswaPayload, MahasiswaDetail,ImportResult},
    repositories::mahasiswa_repo,
};
use axum::{
    extract::{State, Json, Multipart},
    http::StatusCode,
};


/// Handler untuk membuat data Mahasiswa baru, sekaligus membuat akun user-nya.
pub async fn create_mahasiswa_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateMahasiswaPayload>,
) -> Result<(StatusCode, Json<MahasiswaDetail>), AppError> {
    // Memanggil fungsi repository yang sudah kita buat (yang berisi transaksi)
    let created_mahasiswa = mahasiswa_repo::create_mahasiswa_repo(&pool, payload).await?;

    Ok((StatusCode::CREATED, Json(created_mahasiswa)))
}

/// Handler untuk mendapatkan semua data Mahasiswa.
pub async fn get_all_mahasiswa_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<MahasiswaDetail>>, AppError> {
    let mahasiswa_list = mahasiswa_repo::get_all_mahasiswa_repo(&pool).await?;
    Ok(Json(mahasiswa_list))
}

// Anda bisa menambahkan handler lain di sini nanti, seperti get_by_id, update, dan delete
// dengan pola yang sama seperti pada dosen_handler.

pub async fn import_mahasiswa_from_csv_handler(
    State(pool): State<DbPool>,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>, AppError> {
    // Cari field file dari request multipart
    if let Some(field) = multipart.next_field().await? {
        // Pastikan fieldnya adalah 'file'
        if field.name() == Some("file") {
            let file_data = field.bytes().await?;
            let result = mahasiswa_repo::import_mahasiswa_from_csv_repo(&pool, file_data).await?;
            return Ok(Json(result));
        }
    }

    // Jika tidak ada field 'file'
    Err(anyhow::anyhow!("Request harus menyertakan field 'file' dalam format multipart/form-data").into())
}