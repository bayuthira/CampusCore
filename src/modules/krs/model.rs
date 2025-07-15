// src/models/krs_model.rs

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Clone)]
#[sqlx(type_name = "EnrollmentStatus", rename_all = "PascalCase")]
pub enum EnrollmentStatus {
    MenungguPersetujuan,
    Disetujui,
    Ditolak,
    Selesai,
    Mengulang,
}


#[derive(Debug, Deserialize)]
pub struct CreateEnrollmentPayload {
    pub matakuliah_id: Uuid,
    pub tahun_akademik_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct KrsQuery {
    pub tahun_akademik_id: Uuid,
}

// Struct UNTUK MEMBACA DARI DB.
// Field dari tabel join dibuat opsional untuk memenuhi ekspektasi sqlx.
#[derive(Debug, FromRow)]
pub struct EnrollmentFromDb {
    pub id: Uuid,
    pub mahasiswa_id: Uuid,
    pub tahun_akademik: Option<String>,  // Dari LEFT JOIN
    pub kode_mk: Option<String>,         // Dari LEFT JOIN
    pub nama_mk: Option<String>,         // Dari LEFT JOIN
    pub sks: Option<i32>,               // Dari LEFT JOIN
    pub status_approval: String,         // Dari tabel utama, tidak null
    pub nilai_huruf: Option<String>,     // Bisa null
}
// Struct UNTUK DIKIRIM KE FRONTEND
#[derive(Debug, Serialize, FromRow)]
pub struct EnrollmentDetail {
    pub id: Uuid,
    pub mahasiswa_id: Uuid,
    pub tahun_akademik: String,
    pub kode_mk: String,
    pub nama_mk: String,
    pub sks: i32,
    #[sqlx(rename = "status_approval")]
    pub status_approval: EnrollmentStatus,
    pub nilai_huruf: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEnrollmentStatusPayload {
    pub status_approval: EnrollmentStatus,
}

// Implementasi konversi DARI struct DB KE struct Frontend
impl From<EnrollmentFromDb> for EnrollmentDetail {
    fn from(e: EnrollmentFromDb) -> Self {
        let status = match e.status_approval.as_str() {
            "Disetujui" => EnrollmentStatus::Disetujui,
            "Ditolak" => EnrollmentStatus::Ditolak,
            "Selesai" => EnrollmentStatus::Selesai,
            "Mengulang" => EnrollmentStatus::Mengulang,
            _ => EnrollmentStatus::MenungguPersetujuan,
        };

        Self {
            id: e.id,
            mahasiswa_id: e.mahasiswa_id,
            tahun_akademik: e.tahun_akademik.unwrap_or_else(|| "Unknown".to_string()),
            kode_mk: e.kode_mk.unwrap_or_else(|| "Unknown".to_string()),
            nama_mk: e.nama_mk.unwrap_or_else(|| "Unknown".to_string()),
            sks: e.sks.unwrap_or(0),
            status_approval: status,
            nilai_huruf: e.nilai_huruf,
        }
    }
}