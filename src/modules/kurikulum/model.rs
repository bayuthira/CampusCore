// src/modules/kurikulum/model.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct KurikulumDetail {
    pub id: Uuid,
    pub nama: String,
    pub tahun_mulai: i16,
    pub is_active: bool,
    pub prodi_id: Uuid,
    pub nama_prodi: String, // Dari join

    // --- TAMBAHAN FEEDER ---
    pub id_kurikulum_feeder: Option<Uuid>,
    pub sks_lulus: i32,
    pub sks_wajib: i32,
    pub sks_pilihan: i32,
    pub id_semester_mulai: Option<String>,

    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateKurikulumPayload {
    pub nama: String,
    pub tahun_mulai: i16,
    pub is_active: bool,
    pub prodi_id: Uuid,

    // --- TAMBAHAN FEEDER ---
    pub id_kurikulum_feeder: Option<Uuid>,
    pub sks_lulus: Option<i32>, // Dibuat option agar default DB bekerja
    pub sks_wajib: Option<i32>,
    pub sks_pilihan: Option<i32>,
    pub id_semester_mulai: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKurikulumPayload {
    pub nama: Option<String>,
    pub tahun_mulai: Option<i16>,
    pub is_active: Option<bool>,
    pub prodi_id: Option<Uuid>,

    // --- TAMBAHAN FEEDER ---
    pub id_kurikulum_feeder: Option<Uuid>,
    pub sks_lulus: Option<i32>,
    pub sks_wajib: Option<i32>,
    pub sks_pilihan: Option<i32>,
    pub id_semester_mulai: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMataKuliahToKurikulumPayload {
    pub matakuliah_id: Uuid,
}

// --- MODEL UNTUK MEMBACA BARIS CSV ---
#[derive(Debug, Deserialize)]
pub struct MappingCsvRow {
    pub nama_kurikulum: String,
    pub kode_mk: String,
}

// --- MODEL UNTUK RESPONSE HASIL IMPORT ---
#[derive(Debug, Serialize)]
pub struct ImportMappingResponse {
    pub message: String,
    pub success_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>, // Menampilkan alasan kenapa gagal (misal: "Kode MK tidak ditemukan")
}
