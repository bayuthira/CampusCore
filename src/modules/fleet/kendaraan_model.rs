use serde::{Deserialize, Serialize};
use sqlx::Type;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "JenisKendaraan")]
pub enum JenisKendaraan { Mobil, Motor, Bus }

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "StatusKendaraan")]
pub enum StatusKendaraan { Tersedia, Digunakan, Perawatan }

impl JenisKendaraan {
    pub fn as_str(&self) -> &'static str {
        match self {
            JenisKendaraan::Mobil => "Mobil",
            JenisKendaraan::Motor => "Motor",
            JenisKendaraan::Bus => "Bus",
        }
    }
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Kendaraan {
    pub id: Uuid,
    pub jenis: JenisKendaraan,
    pub nama: String,
    pub nomor_polisi: String,
    pub merk: Option<String>,
    pub model: Option<String>,
    pub tahun: Option<i16>,
    pub status: StatusKendaraan,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct KendaraanPayload {
    pub jenis: JenisKendaraan,
    pub nama: String,
    pub nomor_polisi: String,
    pub merk: Option<String>,
    pub model: Option<String>,
    pub tahun: Option<i16>,
}

impl StatusKendaraan {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatusKendaraan::Tersedia => "Tersedia",
            StatusKendaraan::Digunakan => "Digunakan",
            StatusKendaraan::Perawatan => "Perawatan",
        }
    }
}