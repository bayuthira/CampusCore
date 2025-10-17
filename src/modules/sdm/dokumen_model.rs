// src/modules/sdm/dokumen_model.rs
use serde::{Deserialize, Serialize};
use sqlx::Type;
use time::OffsetDateTime;
use uuid::Uuid;

// --- ENUM untuk Entitas Induk ---
#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "SdmEntityType")]
pub enum SdmEntityType {
    Pegawai,
    RiwayatPendidikan,
    RiwayatSk,
}

impl SdmEntityType {
    // Helper untuk mengubah string dari URL menjadi enum
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pegawai" => Some(Self::Pegawai),
            "riwayat-pendidikan" => Some(Self::RiwayatPendidikan),
            "riwayat-sk" => Some(Self::RiwayatSk),
            _ => None,
        }
    }
    
    // Helper untuk mengubah enum menjadi string untuk database
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pegawai => "Pegawai",
            Self::RiwayatPendidikan => "RiwayatPendidikan",
            Self::RiwayatSk => "RiwayatSk",
        }
    }
}

// --- ENUM untuk Kategori Dokumen ---
#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "KategoriDokumen")]
pub enum KategoriDokumen {
    FotoProfil,
    KTP,
    KK,
    Ijazah,
    Transkrip,
    SK,
    Sertifikat,
    Lainnya,
}

impl KategoriDokumen {
    // Helper untuk mengubah string dari form menjadi enum
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "FotoProfil" => Some(Self::FotoProfil),
            "KTP" => Some(Self::KTP),
            "KK" => Some(Self::KK),
            "Ijazah" => Some(Self::Ijazah),
            "Transkrip" => Some(Self::Transkrip),
            "SK" => Some(Self::SK),
            "Sertifikat" => Some(Self::Sertifikat),
            _ => Some(Self::Lainnya),
        }
    }

    // Helper untuk mengubah enum menjadi string untuk database
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FotoProfil => "FotoProfil",
            Self::KTP => "KTP",
            Self::KK => "KK",
            Self::Ijazah => "Ijazah",
            Self::Transkrip => "Transkrip",
            Self::SK => "SK",
            Self::Sertifikat => "Sertifikat",
            Self::Lainnya => "Lainnya",
        }
    }
}

// --- Struct untuk Respons GET ---
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DokumenSdmDetail {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: SdmEntityType,
    pub kategori: KategoriDokumen,
    pub nama_file_asli: String,
    pub path_file: String,
    pub tipe_mime: Option<String>,
    pub user_uploader_id: Uuid,
    pub nama_uploader: String, // dari join
    pub created_at: OffsetDateTime,
}