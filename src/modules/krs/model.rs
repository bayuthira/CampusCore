// src/modules/krs/model.rs

use rust_decimal::Decimal; // <-- Tambahkan untuk tipe NUMERIC SQL
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
#[derive(Debug, FromRow)]
pub struct EnrollmentFromDb {
    pub id: Uuid,
    pub registrasi_id: Uuid, // <-- PERBAIKAN: Menggunakan registrasi_id
    pub tahun_akademik: Option<String>,
    pub kode_mk: Option<String>,
    pub nama_mk: Option<String>,
    pub sks: Option<i32>,
    pub status_approval: String,
    pub nilai_huruf: Option<String>,

    // --- TAMBAHAN FEEDER ---
    pub id_peserta_kelas_feeder: Option<Uuid>,
    pub id_nilai_feeder: Option<Uuid>,
    pub nilai_angka: Option<Decimal>,
    pub nilai_indeks: Option<Decimal>,
}

// Struct UNTUK DIKIRIM KE FRONTEND
#[derive(Debug, Serialize, FromRow)]
pub struct EnrollmentDetail {
    pub id: Uuid,
    pub registrasi_id: Uuid, // <-- PERBAIKAN: Menggunakan registrasi_id
    pub tahun_akademik: String,
    pub kode_mk: String,
    pub nama_mk: String,
    pub sks: i32,
    #[sqlx(rename = "status_approval")]
    pub status_approval: EnrollmentStatus,
    pub nilai_huruf: Option<String>,

    // --- TAMBAHAN FEEDER ---
    pub id_peserta_kelas_feeder: Option<Uuid>,
    pub id_nilai_feeder: Option<Uuid>,
    pub nilai_angka: Option<Decimal>,
    pub nilai_indeks: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEnrollmentStatusPayload {
    pub status_approval: EnrollmentStatus,
}

// --- PAYLOAD BARU UNTUK INPUT NILAI ---
#[derive(Debug, Deserialize)]
pub struct UpdateNilaiPayload {
    pub nilai_angka: Option<Decimal>,
    pub nilai_indeks: Option<Decimal>,
    pub nilai_huruf: Option<String>,
    pub id_nilai_feeder: Option<Uuid>,
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
            registrasi_id: e.registrasi_id, // <-- PERBAIKAN
            tahun_akademik: e.tahun_akademik.unwrap_or_else(|| "Unknown".to_string()),
            kode_mk: e.kode_mk.unwrap_or_else(|| "Unknown".to_string()),
            nama_mk: e.nama_mk.unwrap_or_else(|| "Unknown".to_string()),
            sks: e.sks.unwrap_or(0),
            status_approval: status,
            nilai_huruf: e.nilai_huruf,
            id_peserta_kelas_feeder: e.id_peserta_kelas_feeder,
            id_nilai_feeder: e.id_nilai_feeder,
            nilai_angka: e.nilai_angka,
            nilai_indeks: e.nilai_indeks,
        }
    }
}
