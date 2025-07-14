// src/models/jenis_aset_model.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "KelompokAset", rename_all = "PascalCase")]
pub enum KelompokAset {
    Sarana,
    Prasarana,
}

impl KelompokAset {
    pub fn as_str(&self) -> &'static str {
        match self {
            KelompokAset::Sarana => "Sarana",
            KelompokAset::Prasarana => "Prasarana",
        }
    }
}

// Struct perantara untuk membaca dari database
#[derive(FromRow)]
pub struct JenisAsetFromDb {
    id: Uuid,
    nama_jenis: String,
    deskripsi: Option<String>,
    kelompok: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

// Struct final untuk respons API
#[derive(Debug, Serialize)]
pub struct JenisAset {
    pub id: Uuid,
    pub nama_jenis: String,
    pub deskripsi: Option<String>,
    pub kelompok: KelompokAset,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// Konversi dari struct DB ke struct API
impl From<JenisAsetFromDb> for JenisAset {
    fn from(db_item: JenisAsetFromDb) -> Self {
        let kelompok = match db_item.kelompok.as_str() {
            "Prasarana" => KelompokAset::Prasarana,
            _ => KelompokAset::Sarana, // Default ke Sarana
        };
        Self {
            id: db_item.id,
            nama_jenis: db_item.nama_jenis,
            deskripsi: db_item.deskripsi,
            kelompok,
            created_at: db_item.created_at,
            updated_at: db_item.updated_at,
        }
    }
}

// Payload (tidak berubah)
#[derive(Debug, Deserialize)]
pub struct JenisAsetPayload {
    pub nama_jenis: String,
    pub deskripsi: Option<String>,
    pub kelompok: KelompokAset,
}