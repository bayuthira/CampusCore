use super::model::{RiwayatPendidikan, RiwayatPendidikanPayload};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

pub async fn create_riwayat_pendidikan_repo(pool: &DbPool, pegawai_id: Uuid, payload: RiwayatPendidikanPayload) -> Result<RiwayatPendidikan, AppError> {
    let item = sqlx::query_as!(
        RiwayatPendidikan,
        "INSERT INTO riwayat_pendidikan (pegawai_id, jenjang, institusi, jurusan, tahun_lulus) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        pegawai_id, payload.jenjang, payload.institusi, payload.jurusan, payload.tahun_lulus
    ).fetch_one(pool).await?;
    Ok(item)
}

pub async fn get_all_riwayat_pendidikan_by_pegawai_id_repo(pool: &DbPool, pegawai_id: Uuid) -> Result<Vec<RiwayatPendidikan>, AppError> {
    let list = sqlx::query_as!(
        RiwayatPendidikan,
        "SELECT * FROM riwayat_pendidikan WHERE pegawai_id = $1 ORDER BY tahun_lulus DESC",
        pegawai_id
    ).fetch_all(pool).await?;
    Ok(list)
}

pub async fn update_riwayat_pendidikan_repo(pool: &DbPool, id: Uuid, payload: RiwayatPendidikanPayload) -> Result<RiwayatPendidikan, AppError> {
    let item = sqlx::query_as!(
        RiwayatPendidikan,
        "UPDATE riwayat_pendidikan SET jenjang = $1, institusi = $2, jurusan = $3, tahun_lulus = $4 WHERE id = $5 RETURNING *",
        payload.jenjang, payload.institusi, payload.jurusan, payload.tahun_lulus, id
    ).fetch_one(pool).await?;
    Ok(item)
}

pub async fn delete_riwayat_pendidikan_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM riwayat_pendidikan WHERE id = $1", id)
        .execute(pool).await?.rows_affected();
    if rows_affected == 0 { return Err(sqlx::Error::RowNotFound.into()); }
    Ok(())
}