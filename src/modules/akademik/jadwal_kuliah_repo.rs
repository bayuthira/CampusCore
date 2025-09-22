// src/modules/akademik/jadwal_kuliah_repo.rs
use super::jadwal_kuliah_model::{CreateJadwalKuliahPayload,PlotJadwalRuanganPayload};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;
use time::{Duration, Weekday};


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

pub async fn plot_jadwal_ruangan_repo(
    pool: &DbPool,
    user_pembuat_id: Uuid,
    payload: PlotJadwalRuanganPayload,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    // 1. Ambil detail jadwal kuliah dan tahun akademiknya
    let jadwal = sqlx::query!(
        r#"
        SELECT jk.hari::TEXT as hari, jk.jam_mulai, jk.jam_selesai,
               mk.nama_mk, ta.tanggal_mulai, ta.tanggal_selesai
        FROM jadwal_kuliah jk
        JOIN mata_kuliah mk ON jk.matakuliah_id = mk.id
        JOIN tahun_akademik ta ON jk.tahun_akademik_id = ta.id
        WHERE jk.id = $1
        "#,
        payload.jadwal_kuliah_id
    ).fetch_one(&mut *tx).await?;

    // 2. Hitung semua tanggal pertemuan selama satu semester
    let mut instances_to_create = Vec::new();
    let mut current_date = jadwal.tanggal_mulai;
    let target_weekday = match jadwal.hari.as_deref() {
        Some("Senin") => Weekday::Monday,
        Some("Selasa") => Weekday::Tuesday,
        Some("Rabu") => Weekday::Wednesday,
        Some("Kamis") => Weekday::Thursday,
        Some("Jumat") => Weekday::Friday,
        Some("Sabtu") => Weekday::Saturday,
        _ => Weekday::Sunday,
    };

    while current_date <= jadwal.tanggal_selesai {
        if current_date.weekday() == target_weekday {
            let waktu_mulai = current_date.with_time(jadwal.jam_mulai).assume_utc();
            let waktu_selesai = current_date.with_time(jadwal.jam_selesai).assume_utc();
            instances_to_create.push((waktu_mulai, waktu_selesai));
        }
        current_date += Duration::days(1);
    }

    // 3. Cek konflik & insert semua instance
    for (start_time, end_time) in instances_to_create {
        let conflict = sqlx::query_scalar!(
            "SELECT EXISTS (SELECT 1 FROM jadwal_ruangan WHERE ruangan_id = $1 AND (waktu_mulai, waktu_selesai) OVERLAPS ($2, $3))",
            payload.ruangan_id, start_time, end_time
        ).fetch_one(&mut *tx).await?;

        if conflict.unwrap_or(false) {
            return Err(AppError::Forbidden(format!("Konflik jadwal untuk ruangan pada {}.", start_time.date())));
        }

        sqlx::query!(
            "INSERT INTO jadwal_ruangan (ruangan_id, judul_kegiatan, jadwal_kuliah_id, waktu_mulai, waktu_selesai, user_pembuat_id) VALUES ($1, $2, $3, $4, $5, $6)",
            payload.ruangan_id, jadwal.nama_mk, payload.jadwal_kuliah_id, start_time, end_time, user_pembuat_id
        ).execute(&mut *tx).await?;
    }

    tx.commit().await?;
    Ok(())
}