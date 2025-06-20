// src/models/dosen_model.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Payload untuk membuat Dosen baru
#[derive(Debug, Deserialize)]
pub struct CreateDosenPayload {
    pub nidn: String,
    pub nama_dosen: String,
    pub email: Option<String>, // Email bisa jadi opsional
    pub prodi_id: Uuid,      // WAJIB ada saat membuat dosen baru
}

// Struct ini digunakan untuk menampilkan detail Dosen,
// termasuk nama prodinya (hasil dari JOIN)
#[derive(Debug, Serialize, FromRow)]
pub struct DosenDetail {
    pub id: Uuid,
    pub nidn: String,
    pub nama_dosen: String,
    pub email: Option<String>,
    pub prodi_id: Uuid,
    pub nama_prodi: String, // Kolom tambahan dari tabel prodi
}

#[derive(Debug, Deserialize)]
pub struct UpdateDosenPayload {
    pub nama_dosen: String,
    pub email: Option<String>,
    pub prodi_id: Uuid,
}