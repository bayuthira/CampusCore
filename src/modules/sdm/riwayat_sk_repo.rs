// src/modules/sdm/riwayat_sk_repo.rs
use super::model::{RiwayatSk, RiwayatSkPayload};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

pub async fn create_repo(pool: &DbPool, pegawai_id: Uuid, payload: RiwayatSkPayload) -> Result<RiwayatSk, AppError> {
    let item = sqlx::query_as!(
        RiwayatSk,
        "INSERT INTO riwayat_sk (pegawai_id, nomor_sk, tanggal_sk, jenis_sk, jabatan, keterangan) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        pegawai_id, payload.nomor_sk, payload.tanggal_sk, payload.jenis_sk, payload.jabatan, payload.keterangan
    ).fetch_one(pool).await?;
    Ok(item)
}

pub async fn get_all_by_pegawai_id_repo(pool: &DbPool, pegawai_id: Uuid) -> Result<Vec<RiwayatSk>, AppError> {
    let list = sqlx::query_as!(
        RiwayatSk,
        "SELECT * FROM riwayat_sk WHERE pegawai_id = $1 ORDER BY tanggal_sk DESC",
        pegawai_id
    ).fetch_all(pool).await?;
    Ok(list)
}

pub async fn update_repo(pool: &DbPool, id: Uuid, payload: RiwayatSkPayload) -> Result<RiwayatSk, AppError> {
    let item = sqlx::query_as!(
        RiwayatSk,
        "UPDATE riwayat_sk SET nomor_sk = $1, tanggal_sk = $2, jenis_sk = $3, jabatan = $4, keterangan = $5 WHERE id = $6 RETURNING *",
        payload.nomor_sk, payload.tanggal_sk, payload.jenis_sk, payload.jabatan, payload.keterangan, id
    ).fetch_one(pool).await?;
    Ok(item)
}

pub async fn delete_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM riwayat_sk WHERE id = $1", id)
        .execute(pool).await?.rows_affected();
    if rows_affected == 0 { return Err(sqlx::Error::RowNotFound.into()); }
    Ok(())
}