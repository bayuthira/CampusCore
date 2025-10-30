// src/modules/sdm/riwayat_sertifikat_model.rs
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use time::Date;
use uuid::Uuid;

// --- ENUM ---
#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "KategoriSertifikat")]
pub enum KategoriSertifikat {
    Pelatihan,
    BIMTEK,
    Seminar,
    Workshop,
    #[serde(rename = "Rekognisi Dosen")]
    #[sqlx(rename = "Rekognisi Dosen")]
    RekognisiDosen,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "TingkatSertifikat")]
pub enum TingkatSertifikat {
    Lokal,
    Nasional,
    Internasional,
}

// --- Helper as_str() ---
impl KategoriSertifikat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pelatihan => "Pelatihan",
            Self::BIMTEK => "BIMTEK",
            Self::Seminar => "Seminar",
            Self::Workshop => "Workshop",
            Self::RekognisiDosen => "Rekognisi Dosen",
        }
    }
}
impl TingkatSertifikat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lokal => "Lokal",
            Self::Nasional => "Nasional",
            Self::Internasional => "Internasional",
        }
    }
}

// --- Struct ---
#[derive(Debug, Serialize, FromRow)]
pub struct RiwayatSertifikat {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    pub jenis_sertifikat: KategoriSertifikat,
    pub judul_sertifikat: String,
    pub nomor_sertifikat: Option<String>,
    pub tanggal_pelaksanaan: Date,
    pub tingkat: TingkatSertifikat,
    pub penyelenggara: Option<String>,
    pub keterangan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RiwayatSertifikatPayload {
    pub jenis_sertifikat: KategoriSertifikat,
    pub judul_sertifikat: String,
    pub nomor_sertifikat: Option<String>,
    pub tanggal_pelaksanaan: Date,
    pub tingkat: TingkatSertifikat,
    pub penyelenggara: Option<String>,
    pub keterangan: Option<String>,
}