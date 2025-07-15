// src/models/prodi_model.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow; 
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)] 
pub struct Prodi {
    pub id: Uuid,
    pub kode_prodi: String,
    pub nama_prodi: String,
    
    // Anotasi serde ini sudah bagus untuk menangani JSON
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateProdiPayload {
    pub kode_prodi: String,
    pub nama_prodi: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProdiPayload {
    pub nama_prodi: String,
    pub kode_prodi: Option<String>, // Dibuat opsional
}