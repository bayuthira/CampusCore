// src/modules/sdm/absensi_model.rs
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

// --- ENUM ---

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "StatusAbsensi")]
pub enum StatusAbsensi {
    Hadir,
    Sakit,
    Ijin,
    Cuti,
    Alpa,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "TipeAbsensi")]
pub enum TipeAbsensi {
    ClockIn,
    ClockOut,
}

// Implementasi helper `as_str` untuk Enum
impl StatusAbsensi {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hadir => "Hadir",
            Self::Sakit => "Sakit",
            Self::Ijin => "Ijin",
            Self::Cuti => "Cuti",
            Self::Alpa => "Alpa",
        }
    }
}

impl TipeAbsensi {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClockIn => "ClockIn",
            Self::ClockOut => "ClockOut",
        }
    }
}

// --- MODEL & DTO ---

/// `struct` untuk payload saat Clock-In atau Clock-Out via GPS
#[derive(Debug, Deserialize)]
pub struct ClockPayload {
    pub latitude: Decimal,
    pub longitude: Decimal,
    pub alamat_absensi: Option<String>,
    pub foto_absensi_path: Option<String>,
    pub face_confidence_score: Option<f32>,
    pub is_face_verified: Option<bool>,
}

/// `struct` untuk payload saat Admin membuat rekap manual
#[derive(Debug, Deserialize)]
pub struct RekapManualPayload {
    pub pegawai_id: Uuid,
    pub tanggal: Date,
    pub status: StatusAbsensi,
    pub keterangan: Option<String>,
}

/// `struct` untuk menampilkan data Log Absensi (Respons API)
#[derive(Debug, Serialize, FromRow)]
pub struct LogAbsensi {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub waktu_absensi: OffsetDateTime,
    pub tipe_absensi: TipeAbsensi,
    pub latitude: Decimal,
    pub longitude: Decimal,
    pub alamat_absensi: Option<String>,
    pub foto_absensi_path: Option<String>,
    pub face_confidence_score: Option<f32>, // <-- Diubah menjadi f32
    pub is_face_verified: Option<bool>,
}

/// `struct` untuk menampilkan data Rekap Absensi Harian (Respons API)
#[derive(Debug, Serialize, FromRow)]
pub struct RekapAbsensiHarian {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    pub tanggal: Date,
    pub status: StatusAbsensi,
    pub keterangan: Option<String>,
}

/// `struct` untuk DTO filter rekap absensi
#[derive(Debug, Deserialize)]
pub struct RekapAbsensiFilter {
    pub bulan: i32, // 1-12
    pub tahun: i32,
    pub pegawai_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct LogDayFilter {
    pub tanggal: Date,
}

#[derive(Debug, Deserialize)]
pub struct LaporanHarianFilter {
    pub tanggal: Date,
}

#[derive(Debug, Deserialize)]
pub struct LaporanBulananFilter {
    pub bulan: i32,
    pub tahun: i32,
    pub pegawai_id: Uuid,
}

// Struct raw untuk menerima data dari Database
#[derive(Debug, FromRow)]
pub struct LaporanAbsensiRow {
    pub pegawai_id: Uuid,
    pub nama_pegawai: String,
    pub tanggal: Date,
    pub clock_in: Option<OffsetDateTime>,
    pub clock_out: Option<OffsetDateTime>,
    pub foto_absensi_path_in: Option<String>,
    pub foto_absensi_path_out: Option<String>,
    pub latitude_in: Option<rust_decimal::Decimal>,
    pub longitude_in: Option<rust_decimal::Decimal>,
    pub latitude_out: Option<rust_decimal::Decimal>,
    pub longitude_out: Option<rust_decimal::Decimal>,
    pub status_harian: Option<String>,
    pub ijin_lokasi: Option<String>,
}

// Struct hasil olahan Harian yang akan dikirim ke Frontend
#[derive(Debug, Serialize)]
pub struct LaporanAbsensiResponse {
    pub pegawai_id: Uuid,
    pub nama_pegawai: String,
    pub tanggal: Date,
    #[serde(with = "time::serde::rfc3339::option")]
    pub clock_in: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub clock_out: Option<OffsetDateTime>,
    pub keterangan: String,
    pub terlambat_menit: i32,           // <-- Data angka mentah
    pub terlambat_toleransi_menit: i32, // <-- Data angka mentah
    pub lembur_menit: i32,              // <-- Data angka mentah
    pub foto_absensi_path_in: Option<String>,
    pub foto_absensi_path_out: Option<String>,
    pub latitude_in: Option<rust_decimal::Decimal>,
    pub longitude_in: Option<rust_decimal::Decimal>,
    pub latitude_out: Option<rust_decimal::Decimal>,
    pub longitude_out: Option<rust_decimal::Decimal>,
}

// Struct Khusus Respons Laporan Bulanan (Dengan Total)
#[derive(Debug, Serialize)]
pub struct LaporanBulananResponse {
    pub pegawai_id: Uuid,
    pub nama_pegawai: String,
    pub bulan: i32,
    pub tahun: i32,
    pub total_terlambat_menit: i32, // <-- Total Akumulasi Bulanan
    pub total_terlambat_toleransi_menit: i32, // <-- Total Akumulasi Bulanan
    pub total_lembur_menit: i32,    // <-- Total Akumulasi Bulanan
    pub rekap_harian: Vec<LaporanAbsensiResponse>,
}
