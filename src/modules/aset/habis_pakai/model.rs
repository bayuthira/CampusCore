use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct AsetHabisPakai {
    pub id: Uuid,
    pub nama_barang: String,
    pub deskripsi: Option<String>,
    pub satuan: String,
    pub stok: i32,
    pub batas_minimum_stok: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct AsetHabisPakaiPayload {
    pub nama_barang: String,
    pub deskripsi: Option<String>,
    pub satuan: String,
    pub batas_minimum_stok: i32,
    // Stok awal akan diatur melalui endpoint terpisah (tambah-stok)
}

#[derive(Debug, Deserialize)]
pub struct StokTransaksiPayload {
    pub jumlah: i32, // Jumlah yang ditambah atau diambil
    pub catatan: Option<String>,
    #[serde(default)]
    #[serde(rename = "tanggal_dan_jam", with = "time::serde::rfc3339::option")]
    pub tanggal_transaksi: Option<OffsetDateTime>, // Opsional
}

#[derive(Debug, Serialize, sqlx::Type)]
#[sqlx(type_name = "TipeTransaksiStok")]
pub enum TipeTransaksiStok {
    Pembelian,
    Pengambilan,
     #[sqlx(rename = "Stok Opname")]
    StokOpname,
}

// Struct untuk menampilkan detail histori stok
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct HistoriStokDetail {
    pub id: Uuid,
    pub tipe_transaksi: TipeTransaksiStok,
    pub jumlah: i32,
    pub saldo_sebelum: i32,
    pub saldo_setelah: i32,
    pub catatan: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub tanggal_transaksi: OffsetDateTime,
    pub user_aksi_id: Uuid,
    pub nama_user_aksi: String, // Dari join ke tabel users
}

#[derive(Debug, Deserialize)]
pub struct StokOpnamePayload {
    /// Jumlah stok fisik yang sebenarnya ada saat ini.
    pub stok_fisik: i32,
    pub catatan: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AsetLowStock {
    pub id: Uuid,
    pub nama_barang: String,
    pub stok: i32,
    pub batas_minimum_stok: i32,
}

