// src/models/kurikulum_model.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct KurikulumDetail {
    pub id: Uuid,
    pub nama: String,
    pub tahun_mulai: i16,
    pub is_active: bool,
    pub prodi_id: Uuid,
    pub nama_prodi: String, // Dari join
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateKurikulumPayload {
    pub nama: String,
    pub tahun_mulai: i16,
    pub is_active: bool,
    pub prodi_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKurikulumPayload {
    pub nama: String,
    pub tahun_mulai: i16,
    pub is_active: bool,
    pub prodi_id: Uuid,
}


#[derive(Debug, Deserialize)]
pub struct AddMataKuliahToKurikulumPayload {
    pub matakuliah_id: Uuid,
}