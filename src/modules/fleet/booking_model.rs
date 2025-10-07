use serde::{Deserialize, Serialize};
use sqlx::Type;
use time::OffsetDateTime;
use uuid::Uuid;
use rust_decimal::Decimal;

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

#[derive(Debug, Deserialize)]
pub struct StartTripPayload {
    pub odometer_awal: i32,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub waktu_aktual_berangkat: Option<OffsetDateTime>, // Opsional
}

#[derive(Debug, Deserialize)]
pub struct EndTripPayload {
    pub odometer_akhir: i32,
    pub bahan_bakar_diisi: Option<Decimal>,
    pub catatan_kondisi_kembali: Option<String>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub waktu_aktual_kembali: Option<OffsetDateTime>, // Opsional
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LogPenggunaanDetail {
    pub odometer_awal: Option<i32>,
    pub odometer_akhir: Option<i32>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub waktu_aktual_berangkat: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub waktu_aktual_kembali: Option<OffsetDateTime>,
    pub bahan_bakar_diisi: Option<Decimal>,
    pub catatan_kondisi_kembali: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct BookingSummary {
    pub diajukan: i64,
    pub disetujui: i64,
    pub ditolak: i64,
    pub dibatalkan: i64,
    pub berlangsung: i64,
    pub selesai: i64,
}