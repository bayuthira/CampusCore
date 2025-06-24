// src/models/krs_model.rs

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

// Enum Rust yang MENCERMINKAN ENUM di PostgreSQL
#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Clone)]
#[sqlx(type_name = "EnrollmentStatus", rename_all = "PascalCase")]
pub enum EnrollmentStatus {
    MenungguPersetujuan,
    Disetujui,
    Ditolak,
    Selesai,
    Mengulang,
}

// Payload saat mahasiswa mengambil MK
#[derive(Debug, Deserialize)]
pub struct CreateEnrollmentPayload {
    pub matakuliah_id: Uuid,
    pub tahun_akademik_id: Uuid,
}

// Struct untuk menampilkan detail KRS, hasil dari JOIN banyak tabel
#[derive(Debug, Serialize, FromRow)]
pub struct EnrollmentDetail {
    pub id: Uuid, // id dari tabel enrollments
    pub tahun_akademik: String,
    pub kode_mk: String,
    pub nama_mk: String,
    pub sks: i32,
    #[sqlx(rename = "status_approval")] // Beri tahu sqlx nama kolom aslinya
    pub status_approval: EnrollmentStatus,
    pub nilai_huruf: Option<String>,
}