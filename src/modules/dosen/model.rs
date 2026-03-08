// src/modules/dosen/model.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Payload untuk membuat Dosen baru secara manual (jika tidak via SDM)
// Tidak lagi meminta password/email di sini karena dikelola via Pegawai
#[derive(Debug, Deserialize)]
pub struct CreateDosenPayload {
    pub nidn: String,
    pub pegawai_id: Uuid, // Wajib terhubung ke pegawai
    pub prodi_id: Uuid,

    // --- TAMBAHAN FEEDER ---
    pub id_penugasan_feeder: Option<Uuid>,
    pub ikatan_kerja: Option<String>,
}

// Struct ini digunakan untuk menampilkan detail Dosen
#[derive(Debug, Serialize, FromRow)]
pub struct DosenDetail {
    pub id: Uuid,
    pub nidn: String,
    pub nama_dosen: String,    // Didapat dari JOIN pegawai.nama_lengkap
    pub email: Option<String>, // Didapat dari JOIN pegawai.email
    pub prodi_id: Uuid,
    pub nama_prodi: String,
    pub pegawai_id: Uuid,

    // --- TAMBAHAN FEEDER ---
    pub id_penugasan_feeder: Option<Uuid>,
    pub ikatan_kerja: Option<String>,
}

// Payload untuk Update (menggunakan Option agar bisa Partial Update)
#[derive(Debug, Deserialize)]
pub struct UpdateDosenPayload {
    pub nidn: Option<String>,
    pub prodi_id: Option<Uuid>,

    // --- TAMBAHAN FEEDER ---
    pub id_penugasan_feeder: Option<Uuid>,
    pub ikatan_kerja: Option<String>,
}
