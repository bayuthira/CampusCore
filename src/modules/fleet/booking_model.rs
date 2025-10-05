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

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BookingDetail {
    pub id: Uuid,
    pub kendaraan_id: Uuid,
    pub nama_kendaraan: String,
    pub user_pemesan_id: Uuid,
    pub nama_pemesan: String,
    pub tujuan: String,
    #[serde(with = "time::serde::rfc3339")]
    pub waktu_berangkat: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub estimasi_waktu_kembali: OffsetDateTime,
    pub status: StatusBooking,
}

#[derive(Debug, Deserialize)]
pub struct BookingFilter {
    pub status: Option<StatusBooking>,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalPayload {
    pub catatan: Option<String>,
}

impl StatusBooking {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatusBooking::Diajukan => "Diajukan",
            StatusBooking::Disetujui => "Disetujui",
            StatusBooking::Ditolak => "Ditolak",
            StatusBooking::Dibatalkan => "Dibatalkan",
            StatusBooking::Berlangsung => "Berlangsung",
            StatusBooking::Selesai => "Selesai",
        }
    }
}