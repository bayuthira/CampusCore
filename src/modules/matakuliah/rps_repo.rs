// src/modules/matakuliah/rps_repo.rs
use super::rps_model::{
    RpsHeaderDetail, RpsMingguanDetail, UpsertRpsHeaderPayload, UpsertRpsMingguanPayload,
};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

pub async fn get_rps_header_repo(
    pool: &DbPool,
    mata_kuliah_id: Uuid,
) -> Result<Option<RpsHeaderDetail>, AppError> {
    let header = sqlx::query_as!(
        RpsHeaderDetail,
        r#"
        SELECT 
            mata_kuliah_id, deskripsi_singkat, capaian_pembelajaran, 
            pustaka_utama, pustaka_pendukung, matakuliah_syarat, 
            created_at, updated_at
        FROM mata_kuliah_rps
        WHERE mata_kuliah_id = $1
        "#,
        mata_kuliah_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(header)
}

pub async fn upsert_rps_header_repo(
    pool: &DbPool,
    mata_kuliah_id: Uuid,
    payload: UpsertRpsHeaderPayload,
) -> Result<RpsHeaderDetail, AppError> {
    let header = sqlx::query_as!(
        RpsHeaderDetail,
        r#"
        INSERT INTO mata_kuliah_rps (
            mata_kuliah_id, deskripsi_singkat, capaian_pembelajaran,
            pustaka_utama, pustaka_pendukung, matakuliah_syarat
        ) VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (mata_kuliah_id) DO UPDATE SET
            deskripsi_singkat = EXCLUDED.deskripsi_singkat,
            capaian_pembelajaran = EXCLUDED.capaian_pembelajaran,
            pustaka_utama = EXCLUDED.pustaka_utama,
            pustaka_pendukung = EXCLUDED.pustaka_pendukung,
            matakuliah_syarat = EXCLUDED.matakuliah_syarat,
            updated_at = now()
        RETURNING 
            mata_kuliah_id, deskripsi_singkat, capaian_pembelajaran, 
            pustaka_utama, pustaka_pendukung, matakuliah_syarat, 
            created_at, updated_at
        "#,
        mata_kuliah_id,
        payload.deskripsi_singkat,
        payload.capaian_pembelajaran,
        payload.pustaka_utama,
        payload.pustaka_pendukung,
        payload.matakuliah_syarat
    )
    .fetch_one(pool)
    .await?;

    Ok(header)
}

pub async fn get_rps_mingguan_repo(
    pool: &DbPool,
    mata_kuliah_id: Uuid,
) -> Result<Vec<RpsMingguanDetail>, AppError> {
    let list = sqlx::query_as!(
        RpsMingguanDetail,
        r#"
        SELECT 
            id, mata_kuliah_id, minggu_ke, kemampuan_akhir_diharapkan,
            bahan_kajian, metode_pembelajaran, waktu_belajar,
            kriteria_penilaian, bobot_penilaian
        FROM mata_kuliah_rps_mingguan
        WHERE mata_kuliah_id = $1
        ORDER BY minggu_ke ASC
        "#,
        mata_kuliah_id
    )
    .fetch_all(pool)
    .await?;

    Ok(list)
}

pub async fn upsert_rps_mingguan_repo(
    pool: &DbPool,
    mata_kuliah_id: Uuid,
    payload: UpsertRpsMingguanPayload,
) -> Result<RpsMingguanDetail, AppError> {
    // Upsert berdasarkan kombinasi unique (mata_kuliah_id + minggu_ke)
    let mingguan = sqlx::query_as!(
        RpsMingguanDetail,
        r#"
        INSERT INTO mata_kuliah_rps_mingguan (
            mata_kuliah_id, minggu_ke, kemampuan_akhir_diharapkan,
            bahan_kajian, metode_pembelajaran, waktu_belajar,
            kriteria_penilaian, bobot_penilaian
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (mata_kuliah_id, minggu_ke) DO UPDATE SET
            kemampuan_akhir_diharapkan = EXCLUDED.kemampuan_akhir_diharapkan,
            bahan_kajian = EXCLUDED.bahan_kajian,
            metode_pembelajaran = EXCLUDED.metode_pembelajaran,
            waktu_belajar = EXCLUDED.waktu_belajar,
            kriteria_penilaian = EXCLUDED.kriteria_penilaian,
            bobot_penilaian = EXCLUDED.bobot_penilaian,
            updated_at = now()
        RETURNING 
            id, mata_kuliah_id, minggu_ke, kemampuan_akhir_diharapkan,
            bahan_kajian, metode_pembelajaran, waktu_belajar,
            kriteria_penilaian, bobot_penilaian
        "#,
        mata_kuliah_id,
        payload.minggu_ke,
        payload.kemampuan_akhir_diharapkan,
        payload.bahan_kajian,
        payload.metode_pembelajaran,
        payload.waktu_belajar,
        payload.kriteria_penilaian,
        payload.bobot_penilaian
    )
    .fetch_one(pool)
    .await?;

    Ok(mingguan)
}

pub async fn delete_rps_mingguan_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM mata_kuliah_rps_mingguan WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }

    Ok(())
}
