// src/models/aset_model.rs
use serde::{Deserialize, Serialize};
use sqlx::{Type};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

// ENUM baru untuk kondisi aset
#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "KondisiAset")] // Atribut sqlx tidak berubah
pub enum KondisiAset {
    Baik,
    // Beri tahu serde nama string yang tepat untuk varian ini
    #[serde(rename = "Rusak Ringan")]
    RusakRingan,
    #[serde(rename = "Rusak Berat")]
    RusakBerat,
    #[serde(rename = "Dalam Perbaikan")]
    DalamPerbaikan,
    Dihapuskan,
}

impl KondisiAset {
    pub fn as_str(&self) -> &'static str {
        match self {
            KondisiAset::Baik => "Baik",
            KondisiAset::RusakRingan => "Rusak Ringan",
            KondisiAset::RusakBerat => "Rusak Berat",
            KondisiAset::DalamPerbaikan => "Dalam Perbaikan",
            KondisiAset::Dihapuskan => "Dihapuskan",
        }
    }
}

// Struct untuk respons API
#[derive(Debug, Serialize)]
pub struct AsetDetail {
    pub id: Uuid,
    pub nama_aset: String,
    pub kode_aset: Option<String>,
    pub deskripsi: Option<String>,
    pub tanggal_pembelian: Option<Date>,
    pub kondisi: KondisiAset, // <-- TAMBAHKAN INI
    pub jenis_aset_id: Uuid,
    pub nama_jenis: String,
    pub ruangan_id: Option<Uuid>,
    pub nama_ruangan: Option<String>,
    pub kode_ruangan: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// Payload untuk create dan update
#[derive(Debug, Deserialize)]
pub struct AsetPayload {
    pub nama_aset: String,
    pub kode_aset: Option<String>,
    pub deskripsi: Option<String>,
    pub tanggal_pembelian: Option<Date>,
    pub kondisi: KondisiAset, // <-- TAMBAHKAN INI
    pub jenis_aset_id: Uuid,
    pub ruangan_id: Option<Uuid>,
}


// Enum untuk status histori
#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "AsetHistoriStatus")]
pub enum AsetHistoriStatus {
    Ditempatkan,
    Dipindahkan,
    Dipinjam,
    Dikembalikan,
    #[serde(rename = "Dalam Perbaikan")] // <-- Tambahkan rename untuk serde
    DalamPerbaikan,
    #[serde(rename = "Perbaikan Selesai")]
    PerbaikanSelesai,
    Dihapuskan,
}

impl AsetHistoriStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AsetHistoriStatus::Ditempatkan => "Ditempatkan",
            AsetHistoriStatus::Dipindahkan => "Dipindahkan",
            AsetHistoriStatus::Dipinjam => "Dipinjam",
            AsetHistoriStatus::Dikembalikan => "Dikembalikan",
            AsetHistoriStatus::DalamPerbaikan => "Dalam Perbaikan",
            AsetHistoriStatus::PerbaikanSelesai => "Perbaikan Selesai",
            AsetHistoriStatus::Dihapuskan => "Dihapuskan",
        }
    }
}

// Struct untuk menampilkan detail histori
#[derive(Debug, Serialize)]
pub struct HistoriAsetDetail {
    pub id: Uuid,
    pub status: AsetHistoriStatus,
    pub catatan: String,
    #[serde(with = "time::serde::rfc3339")]
    pub tanggal_kejadian: OffsetDateTime,
    // Informasi user yang melakukan aksi
    pub user_aksi_id: Uuid,
    pub nama_user_aksi: String,
    // Informasi ruangan (opsional)
    pub dari_ruangan: String,
    pub ke_ruangan: String,
}

#[derive(Debug, Deserialize)]
pub struct PindahkanAsetPayload {
    pub ke_ruangan_id: Uuid,
    pub catatan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKondisiPayload {
    pub kondisi: KondisiAset,
    pub catatan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHistoriPayload {
    pub status: AsetHistoriStatus,
    pub catatan: Option<String>,
    pub ke_ruangan_id: Option<Uuid>, // Opsional, hanya diisi saat memindahkan
}

#[derive(Debug, Deserialize)]
pub struct PinjamAsetPayload {
    pub user_peminjam_id: Uuid,
    pub estimasi_tanggal_kembali: OffsetDateTime,
    pub catatan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct KembalikanAsetPayload {
    pub catatan: Option<String>,
}