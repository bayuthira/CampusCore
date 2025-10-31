// src/modules/sdm/riwayat_jad_repo.rs
use super::karir_dosen_model::{
    RiwayatJad, RiwayatJadPayload,
};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

async fn get_by_id_inner(
    executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    id: Uuid,
) -> Result<RiwayatJad, AppError> {
    sqlx::query_as!(
        RiwayatJad,
        r#"
        SELECT 
            id, pegawai_id, 
            jabatan_akademik as "jabatan_akademik: _",
            pangkat_golongan as "pangkat_golongan: _",
            nomor_sk, tmt, kompetensi_mk
        FROM riwayat_jad WHERE id = $1
        "#,
        id
    )
    .fetch_one(executor)
    .await
    .map_err(Into::into)
}

pub async fn create_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
    payload: RiwayatJadPayload,
) -> Result<RiwayatJad, AppError> {
    let ja_str = payload.jabatan_akademik.as_str();
    let pg_str = payload.pangkat_golongan.as_str();

    let id = sqlx::query_scalar(
        r#"
        INSERT INTO riwayat_jad (pegawai_id, jabatan_akademik, pangkat_golongan, nomor_sk, tmt, kompetensi_mk)
        VALUES ($1, $2::"JabatanAkademik", $3::"PangkatGolongan", $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(pegawai_id)
    .bind(ja_str)
    .bind(pg_str)
    .bind(payload.nomor_sk)
    .bind(payload.tmt)
    .bind(payload.kompetensi_mk)
    .fetch_one(pool)
    .await?;

    get_by_id_inner(pool, id).await
}

pub async fn get_all_by_pegawai_id_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
) -> Result<Vec<RiwayatJad>, AppError> {
    sqlx::query_as!(
        RiwayatJad,
        r#"
        SELECT 
            id, pegawai_id, 
            jabatan_akademik as "jabatan_akademik: _",
            pangkat_golongan as "pangkat_golongan: _",
            nomor_sk, tmt, kompetensi_mk
        FROM riwayat_jad WHERE pegawai_id = $1 
        ORDER BY tmt DESC
        "#,
        pegawai_id
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn update_repo(
    pool: &DbPool,
    id: Uuid,
    payload: RiwayatJadPayload,
) -> Result<RiwayatJad, AppError> {
    let ja_str = payload.jabatan_akademik.as_str();
    let pg_str = payload.pangkat_golongan.as_str();

    sqlx::query(
        r#"
        UPDATE riwayat_jad 
        SET jabatan_akademik = $1::"JabatanAkademik", pangkat_golongan = $2::"PangkatGolongan", 
            nomor_sk = $3, tmt = $4, kompetensi_mk = $5
        WHERE id = $6
        "#,
    )
    .bind(ja_str)
    .bind(pg_str)
    .bind(payload.nomor_sk)
    .bind(payload.tmt)
    .bind(payload.kompetensi_mk)
    .bind(id)
    .execute(pool)
    .await?;

    get_by_id_inner(pool, id).await
}

pub async fn delete_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows = sqlx::query!("DELETE FROM riwayat_jad WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();
    if rows == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}