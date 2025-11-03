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