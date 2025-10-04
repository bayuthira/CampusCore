// src/modules/aset/jadwal_ruangan_model.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

// Enum untuk tipe perulangan jadwal
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum TipePerulangan {
    Mingguan,
    Harian,
}

// Struct untuk menampilkan detail jadwal
#[derive(Debug, Serialize, FromRow)]
pub struct JadwalRuangan {
    pub id: Uuid,
    pub ruangan_id: Uuid,
    pub judul_kegiatan: String,
    pub deskripsi: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub waktu_mulai: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub waktu_selesai: OffsetDateTime,
    pub recurring_event_id: Option<Uuid>,
    pub jadwal_kuliah_id: Option<Uuid>,
    pub user_pembuat_id: Uuid,
    pub nama_pembuat: String, // Dari join ke tabel users
}

// Struct untuk payload saat membuat jadwal baru
#[derive(Debug, Deserialize)]
pub struct CreateJadwalPayload {
    pub ruangan_id: Uuid,
    pub judul_kegiatan: String,
    pub deskripsi: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub waktu_mulai: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")] 
    pub waktu_selesai: OffsetDateTime,

    // Field opsional untuk jadwal berulang
    pub tipe_perulangan: Option<TipePerulangan>,
    pub tanggal_akhir_perulangan: Option<Date>,
}

#[derive(Debug, Deserialize)]
pub struct JadwalRuanganFilter {
    #[serde(with = "time::serde::rfc3339")]
    pub start: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub end: OffsetDateTime,
}