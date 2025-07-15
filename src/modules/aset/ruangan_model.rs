// src/models/ruangan_model.rs
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
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct RuanganPayload {
    pub kode_ruangan: String,
    pub nama_ruangan: String,
    pub kapasitas: i32,
}