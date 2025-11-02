// src/modules/sdm/dokumen_model.rs
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use sqlx::{FromRow, Type};

// --- ENUM untuk Entitas Induk ---
#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "SdmEntityType")]
pub enum SdmEntityType {
    Pegawai,
    RiwayatPendidikan,
    RiwayatSk,
    RiwayatSertifikat,
    RiwayatJad,    
    RiwayatSerdos, 
    PengajuanIjin,
}

impl SdmEntityType {
    // Helper untuk mengubah string dari URL menjadi enum
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pegawai" => Some(Self::Pegawai),
            "riwayat-pendidikan" => Some(Self::RiwayatPendidikan),
            "riwayat-sk" => Some(Self::RiwayatSk),
            "riwayat-sertifikat" => Some(Self::RiwayatSertifikat),
            "riwayat-jad" => Some(Self::RiwayatJad),  
            "riwayat-serdos" => Some(Self::RiwayatSerdos), 
            "pengajuan-ijin" => Some(Self::PengajuanIjin),
            _ => None,
        }
    }
    
    // Helper untuk mengubah enum menjadi string untuk database
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pegawai => "Pegawai",
            Self::RiwayatPendidikan => "RiwayatPendidikan",
            Self::RiwayatSk => "RiwayatSk",
            Self::RiwayatSertifikat => "RiwayatSertifikat",
            Self::RiwayatJad => "RiwayatJad",         
            Self::RiwayatSerdos => "RiwayatSerdos",
            Self::PengajuanIjin => "PengajuanIjin",
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
    SuratSakit,
    DokumenPendukung,
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
            "SuratSakit" => Some(Self::SuratSakit),
            "DokumenPendukung" => Some(Self::DokumenPendukung),
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
            Self::SuratSakit => "SuratSakit", 
            Self::DokumenPendukung => "DokumenPendukung",             
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

/// Struct ringan untuk disematkan di list lain (Riwayat SK, dll)
#[derive(Debug, Serialize, FromRow)]
pub struct DokumenSdmSimple {
    pub id: Uuid,
    pub path_file: String,
    #[sqlx(rename = "kategori")] // Ganti nama agar `FromRow` bisa memetakan
    pub kategori: KategoriDokumen,
    pub nama_file_asli: String,
}

#[derive(Debug, Deserialize)]
pub struct DokumenFilter {
    pub pegawai_id: Option<Uuid>,
    pub kategori: Option<KategoriDokumen>,
}

// --- Struct untuk Respons GET ---
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DokumenSdmDetailAll {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    pub nama_pegawai: String,
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