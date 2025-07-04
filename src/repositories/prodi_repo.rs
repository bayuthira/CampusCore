// src/repositories/prodi_repo.rs
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::prodi_model::{CreateProdiPayload, Prodi,UpdateProdiPayload};
use uuid::Uuid;

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

pub async fn get_prodi_by_id_repo(pool: &DbPool, id: Uuid) -> Result<Prodi, AppError> {
    let prodi = sqlx::query_as!(Prodi, "SELECT * FROM prodi WHERE id = $1", id)
        .fetch_one(pool)
        .await?;
    Ok(prodi)
}

pub async fn update_prodi_repo(pool: &DbPool, id: Uuid, payload: UpdateProdiPayload) -> Result<Prodi, AppError> {
    let mut tx = pool.begin().await?;

    // Update nama prodi
    sqlx::query!(
        "UPDATE prodi SET nama_prodi = $1, updated_at = now() WHERE id = $2",
        payload.nama_prodi,
        id
    )
    .execute(&mut *tx)
    .await?;

    // Jika ada kode_prodi baru, update secara terpisah
    if let Some(kode_prodi) = payload.kode_prodi {
        sqlx::query!(
            "UPDATE prodi SET kode_prodi = $1 WHERE id = $2",
            kode_prodi,
            id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let prodi = get_prodi_by_id_repo(pool, id).await?;
    Ok(prodi)
}

pub async fn delete_prodi_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM prodi WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}