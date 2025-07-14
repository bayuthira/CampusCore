// src/models/aset_model.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

// Untuk menampilkan detail aset, hasil dari JOIN
#[derive(Debug, Serialize, FromRow)]
pub struct AsetDetail {
    pub id: Uuid,
    pub nama_aset: String,
    pub kode_aset: Option<String>,
    pub deskripsi: Option<String>,
    pub tanggal_pembelian: Option<Date>,
    pub jenis_aset_id: Uuid,
    pub nama_jenis: String, // dari join
    pub ruangan_id: Option<Uuid>,
    pub nama_ruangan: Option<String>, // dari join
    pub kode_ruangan: Option<String>, // dari join
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
    pub jenis_aset_id: Uuid,
    pub ruangan_id: Option<Uuid>, // Opsional, aset bisa saja di gudang
}