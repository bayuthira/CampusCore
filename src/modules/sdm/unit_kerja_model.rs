// src/modules/sdm/unit_kerja_model.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UnitKerja {
    pub id: Uuid,
    pub induk_unit_id: Option<Uuid>,
    pub kode_unit: Option<String>,
    pub nama_unit: String,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UnitKerjaPayload {
    pub induk_unit_id: Option<Uuid>,
    pub kode_unit: Option<String>,
    pub nama_unit: String,
    pub is_active: Option<bool>,
}