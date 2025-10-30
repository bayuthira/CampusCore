use super::riwayat_sertifikat_model::{
    RiwayatSertifikat, RiwayatSertifikatPayload
};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

async fn get_by_id_inner(executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>, id: Uuid) -> Result<RiwayatSertifikat, AppError> {
    sqlx::query_as!(
        RiwayatSertifikat,
        r#"SELECT id, pegawai_id, 
            jenis_sertifikat as "jenis_sertifikat: _", 
            judul_sertifikat, nomor_sertifikat, tanggal_pelaksanaan, 
            tingkat as "tingkat: _", 
            penyelenggara, keterangan 
        FROM riwayat_sertifikat WHERE id = $1"#,
        id
    ).fetch_one(executor).await.map_err(Into::into)
}

pub async fn create_repo(pool: &DbPool, pegawai_id: Uuid, payload: RiwayatSertifikatPayload) -> Result<RiwayatSertifikat, AppError> {
    let jenis_str = payload.jenis_sertifikat.as_str();
    let tingkat_str = payload.tingkat.as_str();

    let id = sqlx::query_scalar(
        r#"INSERT INTO riwayat_sertifikat (pegawai_id, jenis_sertifikat, judul_sertifikat, nomor_sertifikat, tanggal_pelaksanaan, tingkat, penyelenggara, keterangan) 
           VALUES ($1, $2::"KategoriSertifikat", $3, $4, $5, $6::"TingkatSertifikat", $7, $8) 
           RETURNING id"#,
    )
    .bind(pegawai_id).bind(jenis_str).bind(payload.judul_sertifikat).bind(payload.nomor_sertifikat)
    .bind(payload.tanggal_pelaksanaan).bind(tingkat_str).bind(payload.penyelenggara).bind(payload.keterangan)
    .fetch_one(pool).await?;

    get_by_id_inner(pool, id).await
}

pub async fn get_all_by_pegawai_id_repo(pool: &DbPool, pegawai_id: Uuid) -> Result<Vec<RiwayatSertifikat>, AppError> {
    sqlx::query_as!(
        RiwayatSertifikat,
        r#"SELECT id, pegawai_id, 
            jenis_sertifikat as "jenis_sertifikat: _", 
            judul_sertifikat, nomor_sertifikat, tanggal_pelaksanaan, 
            tingkat as "tingkat: _", 
            penyelenggara, keterangan 
        FROM riwayat_sertifikat WHERE pegawai_id = $1 ORDER BY tanggal_pelaksanaan DESC"#,
        pegawai_id
    ).fetch_all(pool).await.map_err(Into::into)
}

pub async fn update_repo(pool: &DbPool, id: Uuid, payload: RiwayatSertifikatPayload) -> Result<RiwayatSertifikat, AppError> {
    let jenis_str = payload.jenis_sertifikat.as_str();
    let tingkat_str = payload.tingkat.as_str();

    sqlx::query(
        r#"UPDATE riwayat_sertifikat 
           SET jenis_sertifikat = $1::"KategoriSertifikat", judul_sertifikat = $2, nomor_sertifikat = $3, 
               tanggal_pelaksanaan = $4, tingkat = $5::"TingkatSertifikat", penyelenggara = $6, keterangan = $7 
           WHERE id = $8"#,
    )
    .bind(jenis_str).bind(payload.judul_sertifikat).bind(payload.nomor_sertifikat)
    .bind(payload.tanggal_pelaksanaan).bind(tingkat_str).bind(payload.penyelenggara).bind(payload.keterangan)
    .bind(id)
    .execute(pool).await?;

    get_by_id_inner(pool, id).await
}

pub async fn delete_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows = sqlx::query!("DELETE FROM riwayat_sertifikat WHERE id = $1", id)
        .execute(pool).await?.rows_affected();
    if rows == 0 { return Err(sqlx::Error::RowNotFound.into()); }
    Ok(())
}