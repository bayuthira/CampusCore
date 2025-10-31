// src/modules/sdm/cuti_model.rs
use serde::{Deserialize, Serialize};
use sqlx::Type;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

// --- ENUM ---

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "KategoriCuti")]
pub enum KategoriCuti {
    #[serde(rename = "Cuti Tahunan")]
    #[sqlx(rename = "Cuti Tahunan")]
    CutiTahunan,

    #[serde(rename = "Cuti Melahirkan")]
    #[sqlx(rename = "Cuti Melahirkan")]
    CutiMelahirkan,

    #[serde(rename = "Cuti Sakit Berkepanjangan")]
    #[sqlx(rename = "Cuti Sakit Berkepanjangan")]
    CutiSakitBerkepanjangan,

    #[serde(rename = "Cuti Hajatan Keluarga")]
    #[sqlx(rename = "Cuti Hajatan Keluarga")]
    CutiHajatanKeluarga,

    #[serde(rename = "Cuti Ibadah")]
    #[sqlx(rename = "Cuti Ibadah")]
    CutiIbadah,

    Lainnya,
}

impl KategoriCuti {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CutiTahunan => "Cuti Tahunan",
            Self::CutiMelahirkan => "Cuti Melahirkan",
            Self::CutiSakitBerkepanjangan => "Cuti Sakit Berkepanjangan",
            Self::CutiHajatanKeluarga => "Cuti Hajatan Keluarga",
            Self::CutiIbadah => "Cuti Ibadah",
            Self::Lainnya => "Lainnya",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "TipeCuti")]
pub enum TipeCuti {
    Paid,
    Unpaid,
}
impl TipeCuti {
    pub fn as_str(&self) -> &'static str {
        match self {
            TipeCuti::Paid => "Paid",
            TipeCuti::Unpaid => "Unpaid",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "StatusCuti")]
pub enum StatusCuti {
    Diajukan,
    Disetujui,
    Ditolak,
}

// Implementasi helper `as_str` untuk Enum
impl StatusCuti {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatusCuti::Diajukan => "Diajukan",
            StatusCuti::Disetujui => "Disetujui",
            StatusCuti::Ditolak => "Ditolak",
        }
    }
}

// --- MODEL & DTO UNTUK JATAH CUTI ---

/// `struct` untuk menampilkan data Jatah Cuti (Respons API)
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct JatahCuti {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    pub tahun: i16,
    pub kuota_total: i32,
    pub kuota_terpakai: i32,
}

/// `struct` untuk payload saat admin men-generate kuota cuti baru
#[derive(Debug, Deserialize)]
pub struct CreateJatahCutiPayload {
    pub pegawai_id: Uuid,
    pub tahun: i16,
    pub kuota_total: i32,
}

// --- MODEL & DTO UNTUK PENGAJUAN CUTI ---

/// `struct` untuk menampilkan detail Pengajuan Cuti (Respons API)
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PengajuanCuti {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    pub tanggal_mulai: Date,
    pub tanggal_selesai: Date,
    pub jumlah_hari: i32,
    pub alasan: String,
    pub status: StatusCuti,
    pub tipe_cuti: TipeCuti, 
    pub kategori: KategoriCuti,
    pub user_approve_id: Option<Uuid>,
    pub catatan_approval: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreatePengajuanCutiPayload {
    pub tanggal_mulai: Date,
    pub tanggal_selesai: Date,
    pub jumlah_hari: i32,
    pub alasan: String,
    pub kategori: KategoriCuti,
}

/// `struct` untuk payload saat atasan melakukan approval
#[derive(Debug, Deserialize)]
pub struct ApprovalCutiPayload {
    pub catatan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct KuotaFilter {
    pub tahun: i16,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct KuotaCutiDetail {
    pub kuota_total: i32,
    pub kuota_terpakai: i32,
    pub sisa_cuti: i32,
    pub tahun: i16,
}

#[derive(Debug, Deserialize)]
pub struct JatahCutiFilter {
    pub tahun: Option<i16>,
    pub pegawai_id: Option<Uuid>,
}

// Struct baru untuk respons admin, menyertakan nama pegawai
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct JatahCutiDetail {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    pub nama_pegawai: String, // Dari join ke tabel pegawai
    pub nik: String,         // Dari join ke tabel pegawai
    pub tahun: i16,
    pub kuota_total: i32,
    pub kuota_terpakai: i32,
}