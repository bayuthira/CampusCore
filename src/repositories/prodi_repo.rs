// src/repositories/prodi_repo.rs
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::prodi_model::{CreateProdiPayload, Prodi};

// Fungsi untuk membuat prodi baru di database
pub async fn create_prodi_repo(
    pool: &DbPool,
    payload: CreateProdiPayload,
) -> Result<Prodi, AppError> {
    let prodi = sqlx::query_as!(
        Prodi,
        "INSERT INTO prodi (kode_prodi, nama_prodi) VALUES ($1, $2) RETURNING *",
        payload.kode_prodi,
        payload.nama_prodi
    )
    .fetch_one(pool)
    .await?;

    Ok(prodi)
}

// Fungsi untuk mengambil semua prodi dari database
pub async fn get_all_prodi_repo(pool: &DbPool) -> Result<Vec<Prodi>, AppError> {
    let prodi_list = sqlx::query_as!(Prodi, "SELECT * FROM prodi ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;
        
    Ok(prodi_list)
}