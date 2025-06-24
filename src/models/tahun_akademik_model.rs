// src/models/tahun_akademik_model.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

// Struct untuk menampilkan data lengkap
#[derive(Debug, Serialize, FromRow)]
pub struct TahunAkademik {
    pub id: Uuid,
    pub nama: String,
    
    // HAPUS atribut `serde(with)` dari sini. Biarkan default.
    pub tanggal_mulai: Date,
    pub tanggal_selesai: Date,
    pub krs_mulai: Date,
    pub krs_selesai: Date,
    
    pub is_active: bool,
    
    // Atribut untuk OffsetDateTime tetap diperlukan karena formatnya lebih kompleks
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

// Payload untuk membuat dan update
#[derive(Debug, Deserialize)]
pub struct TaPayload {
    pub nama: String,
    
    // HAPUS atribut `serde(with)` dari sini juga.
    pub tanggal_mulai: Date,
    pub tanggal_selesai: Date,
    pub krs_mulai: Date,
    pub krs_selesai: Date,
    
    pub is_active: bool,
}