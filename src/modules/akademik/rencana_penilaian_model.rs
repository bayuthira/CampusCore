// src/modules/akademik/rencana_penilaian_model.rs
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

// Struct untuk Response (Detail Kesiapan Buka Kelas)
#[derive(Debug, Serialize, FromRow)]
pub struct RencanaPenilaianDetail {
    pub id: Uuid,
    pub jadwal_kuliah_id: Uuid,
    pub file_kontrak_path: Option<String>,
    pub bobot_kehadiran: Decimal,
    pub bobot_tugas: Decimal,
    pub bobot_uts: Decimal,
    pub bobot_uas: Decimal,
    pub bobot_praktek: Decimal,
    pub catatan_rencana_praktikum: Option<String>,
    pub file_praktikum_path: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

// Payload untuk Set/Update Bobot dan Catatan (Tanpa file)
#[derive(Debug, Deserialize)]
pub struct UpsertRencanaPenilaianPayload {
    pub bobot_kehadiran: Decimal,
    pub bobot_tugas: Decimal,
    pub bobot_uts: Decimal,
    pub bobot_uas: Decimal,
    pub bobot_praktek: Decimal,
    pub catatan_rencana_praktikum: Option<String>,
}
