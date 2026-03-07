// src/modules/matakuliah/model.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Untuk membuat MK baru
#[derive(Debug, Deserialize)]
pub struct CreateMataKuliahPayload {
    pub kode_mk: String,
    pub nama_mk: String,
    pub semester_target: i32,
    pub prodi_id: Uuid,

    // --- TAMBAHAN FEEDER ---
    pub id_matkul_feeder: Option<Uuid>,
    pub sks_tatap_muka: i32,
    pub sks_praktek: i32,
    pub sks_praktek_lapangan: i32,
    pub sks_simulasi: i32,
    pub jenis_mk: Option<String>, // Jika kosong, otomatis "Wajib"
}

// Untuk update MK (Semuanya Option agar bisa Partial Update)
#[derive(Debug, Deserialize)]
pub struct UpdateMataKuliahPayload {
    pub nama_mk: Option<String>,
    pub semester_target: Option<i32>,
    pub prodi_id: Option<Uuid>,
    pub kode_mk: Option<String>,

    // --- TAMBAHAN FEEDER ---
    pub id_matkul_feeder: Option<Uuid>,
    pub sks_tatap_muka: Option<i32>,
    pub sks_praktek: Option<i32>,
    pub sks_praktek_lapangan: Option<i32>,
    pub sks_simulasi: Option<i32>,
    pub jenis_mk: Option<String>,
}

// Untuk menampilkan detail MK, termasuk nama prodinya
#[derive(Debug, Serialize, FromRow)]
pub struct MataKuliahDetail {
    pub id: Uuid,
    pub kode_mk: String,
    pub nama_mk: String,
    pub sks: i32, // Kolom ini sekarang digunakan sebagai Total SKS
    pub semester_target: i32,
    pub prodi_id: Uuid,
    pub nama_prodi: String, // Dari tabel prodi

    // --- TAMBAHAN FEEDER ---
    pub id_matkul_feeder: Option<Uuid>,
    pub sks_tatap_muka: i32,
    pub sks_praktek: i32,
    pub sks_praktek_lapangan: i32,
    pub sks_simulasi: i32,
    pub jenis_mk: String,
}
