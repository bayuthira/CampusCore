use serde::{Deserialize, Serialize};
use sqlx::Type;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "StatusBooking")]
pub enum StatusBooking { Diajukan, Disetujui, Ditolak, Dibatalkan, Berlangsung, Selesai }

#[derive(Debug, Deserialize)]
pub struct CreateBookingPayload {
    pub kendaraan_id: Uuid,
    pub tujuan: String,
    #[serde(with = "time::serde::rfc3339")]
    pub waktu_berangkat: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub estimasi_waktu_kembali: OffsetDateTime,
}