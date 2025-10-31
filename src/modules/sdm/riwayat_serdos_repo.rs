// src/modules/sdm/riwayat_serdos_repo.rs
use super::karir_dosen_model::{RiwayatSerdos, RiwayatSerdosPayload};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

/// Helper internal untuk mengambil data berdasarkan ID
async fn get_by_id_inner(
    executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    id: Uuid,
) -> Result<RiwayatSerdos, AppError> {
    sqlx::query_as!(
        RiwayatSerdos,
        // PERBAIKAN: Sebutkan kolom secara eksplisit, jangan `SELECT *`
        "SELECT id, pegawai_id, nomor_sertifikat, tanggal_terbit, keterangan 
         FROM riwayat_serdos WHERE id = $1",
        id
    )
    .fetch_one(executor)
    .await
    .map_err(Into::into)
}

/// Membuat data SERDOS baru
pub async fn create_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
    payload: RiwayatSerdosPayload,
) -> Result<RiwayatSerdos, AppError> {
    let id = sqlx::query_scalar!(
        "INSERT INTO riwayat_serdos (pegawai_id, nomor_sertifikat, tanggal_terbit, keterangan)
         VALUES ($1, $2, $3, $4) RETURNING id",
        pegawai_id,
        payload.nomor_sertifikat,
        payload.tanggal_terbit,
        payload.keterangan
    )
    .fetch_one(pool)
    .await?;

    // Panggil helper yang sudah diperbaiki
    get_by_id_inner(pool, id).await
}

/// Mengambil semua data SERDOS milik satu pegawai
pub async fn get_all_by_pegawai_id_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
) -> Result<Vec<RiwayatSerdos>, AppError> {
    sqlx::query_as!(
        RiwayatSerdos,
        // PERBAIKAN: Sebutkan kolom secara eksplisit, jangan `SELECT *`
        "SELECT id, pegawai_id, nomor_sertifikat, tanggal_terbit, keterangan 
         FROM riwayat_serdos WHERE pegawai_id = $1 
         ORDER BY tanggal_terbit DESC",
        pegawai_id
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Memperbarui data SERDOS
pub async fn update_repo(
    pool: &DbPool,
    id: Uuid,
    payload: RiwayatSerdosPayload,
) -> Result<RiwayatSerdos, AppError> {
    sqlx::query!(
        "UPDATE riwayat_serdos 
         SET nomor_sertifikat = $1, tanggal_terbit = $2, keterangan = $3
         WHERE id = $4",
        payload.nomor_sertifikat,
        payload.tanggal_terbit,
        payload.keterangan,
        id
    )
    .execute(pool)
    .await?;

    // Panggil helper yang sudah diperbaiki
    get_by_id_inner(pool, id).await
}

/// Menghapus data SERDOS
pub async fn delete_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows = sqlx::query!("DELETE FROM riwayat_serdos WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();
    if rows == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}