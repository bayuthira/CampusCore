// src/modules/prodi/model.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Prodi {
    pub id: Uuid,
    pub kode_prodi: String,
    pub nama_prodi: String,

    // --- TAMBAHAN KOLOM FEEDER ---
    pub id_prodi_feeder: Option<Uuid>,
    pub jenjang: Option<String>,
    pub status_prodi: Option<String>,

    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateProdiPayload {
    pub kode_prodi: String,
    pub nama_prodi: String,
    // --- TAMBAHAN KOLOM FEEDER ---
    pub id_prodi_feeder: Option<Uuid>,
    pub jenjang: Option<String>,
    pub status_prodi: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProdiPayload {
    pub nama_prodi: Option<String>, // Diubah menjadi Option untuk mendukung partial update
    pub kode_prodi: Option<String>, // Dibuat opsional

    // --- TAMBAHAN KOLOM FEEDER ---
    pub id_prodi_feeder: Option<Uuid>,
    pub jenjang: Option<String>,
    pub status_prodi: Option<String>,
}
