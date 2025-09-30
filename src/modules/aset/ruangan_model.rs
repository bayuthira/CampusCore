// src/models/ruangan_model.rs
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct Ruangan {
    pub id: Uuid,
    pub kode_ruangan: String,
    pub nama_ruangan: String,
    pub kapasitas: i32,
    pub panjang: Decimal,
    pub lebar: Decimal,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct RuanganPayload {
    pub kode_ruangan: String,
    pub nama_ruangan: String,
    pub kapasitas: i32,
    pub panjang: Decimal,
    pub lebar: Decimal,  
}

#[derive(Debug, Deserialize)]
pub struct RuanganFilter {
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RuanganTersediaFilter {
    pub jadwal_kuliah_id: Uuid,
    pub q: Option<String>, // Untuk search nama/kode ruangan
}

// Untuk respons: hanya ID dan nama ruangan
#[derive(Debug, Serialize, FromRow)]
pub struct RuanganLookup {
    pub id: Uuid,
    pub nama_ruangan: String,
    pub kode_ruangan: String,
    pub kapasitas: i32,
}