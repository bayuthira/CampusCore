// src/models/matakuliah_model.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Untuk membuat MK baru
#[derive(Debug, Deserialize)]
pub struct CreateMataKuliahPayload {
    pub kode_mk: String,
    pub nama_mk: String,
    pub sks: i32,
    pub semester_target: i32,
    pub prodi_id: Uuid,
}

// Untuk update MK
#[derive(Debug, Deserialize)]
pub struct UpdateMataKuliahPayload {
    pub nama_mk: String,
    pub sks: i32,
    pub semester_target: i32,
    pub prodi_id: Uuid,
    pub kode_mk: Option<String>,
}

// Untuk menampilkan detail MK, termasuk nama prodinya
#[derive(Debug, Serialize, FromRow)]
pub struct MataKuliahDetail {
    pub id: Uuid,
    pub kode_mk: String,
    pub nama_mk: String,
    pub sks: i32,
    pub semester_target: i32,
    pub prodi_id: Uuid,
    pub nama_prodi: String, // Dari tabel prodi
}