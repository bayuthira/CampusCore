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
    pub tanggal_mulai: Date,
    pub tanggal_selesai: Date,
    pub krs_mulai: Date,
    pub krs_selesai: Date,
    pub is_active: bool,
    pub id_semester_feeder: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

// Payload untuk membuat dan update
#[derive(Debug, Deserialize)]
pub struct TaPayload {
    pub nama: String,
    pub tanggal_mulai: Date,
    pub tanggal_selesai: Date,
    pub krs_mulai: Date,
    pub krs_selesai: Date,
    pub is_active: bool,
    pub id_semester_feeder: Option<String>,
}
