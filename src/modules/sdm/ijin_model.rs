// src/modules/sdm/ijin_model.rs
use serde::{Deserialize, Serialize};
use sqlx::Type;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

// --- ENUM ---

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "KategoriIjin")]
pub enum KategoriIjin {
    Sakit,
    #[serde(rename = "Urusan Keluarga")]
    #[sqlx(rename = "Urusan Keluarga")]
    UrusanKeluarga,
    Lainnya,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "StatusIjin")]
pub enum StatusIjin {
    Diajukan,
    Disetujui,
    Ditolak,
}

// Implementasi helper `as_str` untuk Enum
impl KategoriIjin {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sakit => "Sakit",
            Self::UrusanKeluarga => "Urusan Keluarga",
            Self::Lainnya => "Lainnya",
        }
    }
}

impl StatusIjin {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Diajukan => "Diajukan",
            Self::Disetujui => "Disetujui",
            Self::Ditolak => "Ditolak",
        }
    }
}

// --- MODEL & DTO UNTUK PENGAJUAN IJIN ---

/// `struct` untuk menampilkan detail Pengajuan Ijin (Respons API)
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PengajuanIjin {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    pub kategori: KategoriIjin,
    pub tanggal_mulai: Date,
    pub tanggal_selesai: Date,
    pub alasan: String,
    pub status: StatusIjin,
    pub user_approve_id: Option<Uuid>,
    pub catatan_approval: Option<String>,
    pub created_at: OffsetDateTime,
}

/// `struct` untuk payload saat pegawai membuat pengajuan ijin baru
#[derive(Debug, Deserialize)]
pub struct CreatePengajuanIjinPayload {
    pub kategori: KategoriIjin,
    pub tanggal_mulai: Date,
    pub tanggal_selesai: Date,
    pub alasan: String,
}

/// `struct` untuk payload saat atasan melakukan approval
#[derive(Debug, Deserialize)]
pub struct ApprovalIjinPayload {
    pub catatan: Option<String>,
}