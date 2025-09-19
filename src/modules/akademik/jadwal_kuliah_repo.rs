// src/modules/akademik/jadwal_kuliah_repo.rs
use super::jadwal_kuliah_model::{CreateJadwalKuliahPayload};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;


pub async fn create_jadwal_kuliah_repo(
    pool: &DbPool,
    payload: CreateJadwalKuliahPayload,
) -> Result<Uuid, AppError> {
    let mut tx = pool.begin().await?;

    let hari_str = payload.hari.as_str(); // Konversi enum ke string

    // Gunakan sqlx::query() (tanpa !) dan .bind()
    let jadwal_id = sqlx::query_scalar(
        r#"
        INSERT INTO jadwal_kuliah 
        (matakuliah_id, tahun_akademik_id, hari, jam_mulai, jam_selesai, kelas)
        VALUES ($1, $2, $3::"DayOfWeek", $4, $5, $6) RETURNING id
        "#,
    )
    .bind(payload.matakuliah_id)
    .bind(payload.tahun_akademik_id)
    .bind(hari_str) // Kirim sebagai string
    .bind(payload.jam_mulai)
    .bind(payload.jam_selesai)
    .bind(payload.kelas)
    .fetch_one(&mut *tx)
    .await?;

    for dosen in payload.dosen_pengampu {
        let peran_str = dosen.peran.as_str(); // Konversi enum ke string
        sqlx::query(
            "INSERT INTO jadwal_dosen_pengampu (jadwal_kuliah_id, dosen_id, peran) VALUES ($1, $2, $3::\"PeranDosenPengampu\")",
        )
        .bind(jadwal_id)
        .bind(dosen.dosen_id)
        .bind(peran_str) // Kirim sebagai string
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(jadwal_id)
}