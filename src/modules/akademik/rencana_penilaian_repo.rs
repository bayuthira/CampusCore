// src/modules/akademik/rencana_penilaian_repo.rs
use super::rencana_penilaian_model::{RencanaPenilaianDetail, UpsertRencanaPenilaianPayload};
use crate::{db::DbPool, errors::AppError};
use rust_decimal::Decimal;
use uuid::Uuid;

/// Mengambil data rencana penilaian untuk sebuah jadwal/kelas tertentu
pub async fn get_rencana_penilaian_by_jadwal_repo(
    pool: &DbPool,
    jadwal_kuliah_id: Uuid,
) -> Result<Option<RencanaPenilaianDetail>, AppError> {
    let rencana = sqlx::query_as!(
        RencanaPenilaianDetail,
        r#"
        SELECT 
            id, jadwal_kuliah_id, file_kontrak_path,
            bobot_kehadiran, bobot_tugas, bobot_uts, bobot_uas, bobot_praktek,
            catatan_rencana_praktikum, file_praktikum_path,
            created_at, updated_at
        FROM jadwal_rencana_penilaian
        WHERE jadwal_kuliah_id = $1
        "#,
        jadwal_kuliah_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(rencana)
}

/// Melakukan Insert (Jika Belum Ada) atau Update (Jika Sudah Ada) bobot penilaian
pub async fn upsert_rencana_penilaian_repo(
    pool: &DbPool,
    jadwal_kuliah_id: Uuid,
    payload: UpsertRencanaPenilaianPayload,
) -> Result<RencanaPenilaianDetail, AppError> {
    // --- 1. VALIDASI WAJIB 100% ---
    let total_bobot = payload.bobot_kehadiran
        + payload.bobot_tugas
        + payload.bobot_uts
        + payload.bobot_uas
        + payload.bobot_praktek;

    // Pastikan totalnya persis 100.00
    if total_bobot != Decimal::from(100) {
        return Err(AppError::BadRequest(format!(
            "Total bobot penilaian harus tepat 100%. Saat ini: {}%",
            total_bobot
        )));
    }

    // --- 2. JALANKAN UPSERT (ON CONFLICT DO UPDATE) ---
    let rencana = sqlx::query_as!(
        RencanaPenilaianDetail,
        r#"
        INSERT INTO jadwal_rencana_penilaian (
            jadwal_kuliah_id, bobot_kehadiran, bobot_tugas, bobot_uts, bobot_uas, bobot_praktek, catatan_rencana_praktikum
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (jadwal_kuliah_id) DO UPDATE SET
            bobot_kehadiran = EXCLUDED.bobot_kehadiran,
            bobot_tugas = EXCLUDED.bobot_tugas,
            bobot_uts = EXCLUDED.bobot_uts,
            bobot_uas = EXCLUDED.bobot_uas,
            bobot_praktek = EXCLUDED.bobot_praktek,
            catatan_rencana_praktikum = EXCLUDED.catatan_rencana_praktikum,
            updated_at = now()
        RETURNING 
            id, jadwal_kuliah_id, file_kontrak_path,
            bobot_kehadiran, bobot_tugas, bobot_uts, bobot_uas, bobot_praktek,
            catatan_rencana_praktikum, file_praktikum_path,
            created_at, updated_at
        "#,
        jadwal_kuliah_id,
        payload.bobot_kehadiran,
        payload.bobot_tugas,
        payload.bobot_uts,
        payload.bobot_uas,
        payload.bobot_praktek,
        payload.catatan_rencana_praktikum
    )
    .fetch_one(pool)
    .await?;

    Ok(rencana)
}

/// Fungsi helper untuk mengupdate path file (Kontrak Kuliah / Praktikum) via upload dokumen
pub async fn update_file_rencana_penilaian_repo(
    pool: &DbPool,
    jadwal_kuliah_id: Uuid,
    kolom: &str, // 'file_kontrak_path' atau 'file_praktikum_path'
    file_path: String,
) -> Result<(), AppError> {
    // Pastikan recordnya sudah ada sebelum diupload filenya. Jika belum, error.
    let existing = get_rencana_penilaian_by_jadwal_repo(pool, jadwal_kuliah_id).await?;

    if existing.is_none() {
        return Err(AppError::BadRequest(
            "Anda harus menyusun persentase bobot penilaian terlebih dahulu sebelum mengupload dokumen.".to_string()
        ));
    }

    // Dynamic query update menggunakan format
    let query = format!(
        "UPDATE jadwal_rencana_penilaian SET {} = $1, updated_at = now() WHERE jadwal_kuliah_id = $2",
        kolom
    );

    sqlx::query(&query)
        .bind(file_path)
        .bind(jadwal_kuliah_id)
        .execute(pool)
        .await?;

    Ok(())
}
