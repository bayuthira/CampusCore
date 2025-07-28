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
