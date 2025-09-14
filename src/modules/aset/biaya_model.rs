//src/modules/aset/biaya_model.rs
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

impl Default for TipeBiaya {
    fn default() -> Self {
        // Kita tentukan 'Lain-lain' sebagai nilai default
        TipeBiaya::LainLain
    }
}
// Enum untuk tipe biaya, mencerminkan ENUM di DB
#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "TipeBiaya", rename_all = "PascalCase")]
pub enum TipeBiaya {
    Pembelian,
    Perawatan,
    Perbaikan,
    Upgrade,
    #[serde(rename = "Lain-lain")]
    LainLain,
}

// Struct untuk respons API (menampilkan detail)
#[derive(Debug, Serialize, FromRow)]
pub struct BiayaAset {
    pub id: Uuid,
    pub aset_id: Uuid,
    pub tipe_biaya: TipeBiaya,
    pub deskripsi: String,
    pub jumlah: Decimal,
    pub tanggal_transaksi: Date,
    pub vendor: Option<String>,
    pub user_pencatat_id: Uuid,
    pub nama_pencatat: String, // Dari join ke tabel users
    pub bukti_url: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// Payload untuk create dan update
#[derive(Debug, Deserialize)]
pub struct BiayaAsetPayload {
    pub aset_id: Uuid,
    pub tipe_biaya: TipeBiaya,
    pub deskripsi: String,
    pub jumlah: Decimal,
    pub tanggal_transaksi: Date,
    pub vendor: Option<String>,
}

impl TipeBiaya {
    pub fn as_str(&self) -> &'static str {
        match self {
            TipeBiaya::Pembelian => "Pembelian",
            TipeBiaya::Perawatan => "Perawatan",
            TipeBiaya::Perbaikan => "Perbaikan",
            TipeBiaya::Upgrade => "Upgrade",
            TipeBiaya::LainLain => "Lain-lain",
        }
    }
}