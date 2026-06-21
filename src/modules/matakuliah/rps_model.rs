// src/modules/matakuliah/rps_model.rs
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct RpsMataKuliahAccess {
    pub id: Uuid,
    pub kode_mk: String,
    pub nama_mk: String,
    pub prodi_id: Uuid,
    pub nama_prodi: String,
    pub file_rps_path: Option<String>,
    pub status_verifikasi_rps: Option<String>,
    pub catatan_verifikasi_rps: Option<String>,
    pub peran_pengampu: Option<String>,
    pub can_edit: bool,
    pub can_verify: bool,
}

// ==========================================
// 1. MODEL UNTUK HEADER RPS (1-to-1)
// ==========================================

#[derive(Debug, Serialize, FromRow)]
pub struct RpsHeaderDetail {
    pub mata_kuliah_id: Uuid,
    pub deskripsi_singkat: Option<String>,
    pub capaian_pembelajaran: Option<String>,
    pub pustaka_utama: Option<String>,
    pub pustaka_pendukung: Option<String>,
    pub matakuliah_syarat: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct UpsertRpsHeaderPayload {
    pub deskripsi_singkat: Option<String>,
    pub capaian_pembelajaran: Option<String>,
    pub pustaka_utama: Option<String>,
    pub pustaka_pendukung: Option<String>,
    pub matakuliah_syarat: Option<String>,
}

// ==========================================
// 2. MODEL UNTUK MATRIKS MINGGUAN (1-to-Many)
// ==========================================

#[derive(Debug, Serialize, FromRow)]
pub struct RpsMingguanDetail {
    pub id: Uuid,
    pub mata_kuliah_id: Uuid,
    pub minggu_ke: i32,
    pub kemampuan_akhir_diharapkan: Option<String>,
    pub bahan_kajian: Option<String>,
    pub metode_pembelajaran: Option<String>,
    pub waktu_belajar: Option<String>,
    pub kriteria_penilaian: Option<String>,
    pub bobot_penilaian: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertRpsMingguanPayload {
    pub minggu_ke: i32,
    pub kemampuan_akhir_diharapkan: Option<String>,
    pub bahan_kajian: Option<String>,
    pub metode_pembelajaran: Option<String>,
    pub waktu_belajar: Option<String>,
    pub kriteria_penilaian: Option<String>,
    pub bobot_penilaian: Option<Decimal>,
}
