// src/models/mahasiswa_model.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Payload untuk membuat Mahasiswa baru.
// Perhatikan kita juga meminta password untuk akun user-nya.
#[derive(Debug, Deserialize)]
pub struct CreateMahasiswaPayload {
    pub nim: String,
    pub nama_mahasiswa: String,
    pub email: String,
    pub angkatan: i32,
    pub prodi_id: Uuid,
    pub password: String, // Password awal untuk akun login mahasiswa
}

// Struct untuk menampilkan detail Mahasiswa, hasil dari JOIN beberapa tabel.
#[derive(Debug, Serialize, FromRow)]
pub struct MahasiswaDetail {
    pub id: Uuid, // id dari tabel mahasiswa
    pub nim: String,
    pub nama_mahasiswa: String,
    pub angkatan: i32,
    pub email: Option<String>,
    pub prodi_id: Uuid,
    pub nama_prodi: String,
    pub user_id: Option<Uuid>,
    pub username: Option<String>, // username dari tabel users
}

#[derive(Debug, Deserialize)]
pub struct MahasiswaCsvRecord {
    pub nim: String,
    pub nama_mahasiswa: String,
    pub email: String,
    pub angkatan: i32,
    pub kode_prodi: String, // Kita gunakan kode prodi di CSV agar lebih user-friendly
}

// Struct untuk laporan hasil impor
#[derive(Debug, Serialize)]
pub struct ImportResult {
    // Atribut `rename` dari serde akan mengubah nama field ini di JSON
    #[serde(rename = "data_valid")]
    pub data_valid: u32,

    #[serde(rename = "data_tidak_valid")]
    pub data_tidak_valid: u32,

    #[serde(rename = "rincian_tidak_valid")]
    pub rincian_tidak_valid: Vec<String>,
}