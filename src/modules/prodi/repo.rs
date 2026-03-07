// src/modules/prodi/repo.rs
use super::model::{CreateProdiPayload, Prodi, UpdateProdiPayload};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

// Fungsi untuk membuat prodi baru di database
pub async fn create_prodi_repo(
    pool: &DbPool,
    payload: CreateProdiPayload,
) -> Result<Prodi, AppError> {
    // Default status_prodi ke 'Aktif' jika tidak dikirim dari FE
    let status = payload.status_prodi.unwrap_or_else(|| "Aktif".to_string());

    let prodi = sqlx::query_as!(
        Prodi,
        r#"
        INSERT INTO prodi (kode_prodi, nama_prodi, id_prodi_feeder, jenjang, status_prodi) 
        VALUES ($1, $2, $3, $4, $5) RETURNING *
        "#,
        payload.kode_prodi,
        payload.nama_prodi,
        payload.id_prodi_feeder,
        payload.jenjang,
        status
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

pub async fn update_prodi_repo(
    pool: &DbPool,
    id: Uuid,
    payload: UpdateProdiPayload,
) -> Result<Prodi, AppError> {
    let mut tx = pool.begin().await?;

    // Ambil data lama untuk mendukung partial update
    let old_prodi = get_prodi_by_id_repo(pool, id).await?;

    let upd_nama = payload.nama_prodi.unwrap_or(old_prodi.nama_prodi);
    let upd_kode = payload.kode_prodi.unwrap_or(old_prodi.kode_prodi);
    let upd_feeder = payload.id_prodi_feeder.or(old_prodi.id_prodi_feeder);
    let upd_jenjang = payload.jenjang.or(old_prodi.jenjang);
    let upd_status = payload.status_prodi.or(old_prodi.status_prodi);

    // Lakukan satu update query utuh
    sqlx::query!(
        r#"
        UPDATE prodi 
        SET nama_prodi = $1, 
            kode_prodi = $2, 
            id_prodi_feeder = $3, 
            jenjang = $4, 
            status_prodi = $5, 
            updated_at = now() 
        WHERE id = $6
        "#,
        upd_nama,
        upd_kode,
        upd_feeder,
        upd_jenjang,
        upd_status,
        id
    )
    .execute(&mut *tx)
    .await?;

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
