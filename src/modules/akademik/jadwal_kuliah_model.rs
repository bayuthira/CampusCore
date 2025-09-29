// src/modules/akademik/jadwal_kuliah_model.rs
use serde::{Deserialize, Deserializer,Serialize};
use time::{format_description::FormatItem, macros::format_description, Time};
use uuid::Uuid;

// Modul helper kustom HANYA untuk deserialize
mod time_format_hm {
    use super::*;

    const FORMAT: &[FormatItem<'_>] = format_description!("[hour]:[minute]");

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Time, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Time::parse(&s, &FORMAT).map_err(serde::de::Error::custom)
    }
}

// Enum untuk mencerminkan ENUM di DB
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq)]
#[sqlx(type_name = "DayOfWeek")]
pub enum DayOfWeek {
    Senin,
    Selasa,
    Rabu,
    Kamis,
    Jumat,
    Sabtu,
    Minggu,
}

impl DayOfWeek {
    pub fn as_str(&self) -> &'static str {
        match self {
            DayOfWeek::Senin => "Senin",
            DayOfWeek::Selasa => "Selasa",
            DayOfWeek::Rabu => "Rabu",
            DayOfWeek::Kamis => "Kamis",
            DayOfWeek::Jumat => "Jumat",
            DayOfWeek::Sabtu => "Sabtu",
            DayOfWeek::Minggu => "Minggu",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq)]
#[sqlx(type_name = "PeranDosenPengampu")]
pub enum PeranDosenPengampu {
    Koordinator,
    Anggota,
}

#[derive(Debug, Deserialize)]
pub struct DosenPengampuPayload {
    pub dosen_id: Uuid,
    pub peran: PeranDosenPengampu,
}

impl PeranDosenPengampu {
    pub fn as_str(&self) -> &'static str {
        match self {
            PeranDosenPengampu::Koordinator => "Koordinator",
            PeranDosenPengampu::Anggota => "Anggota",
        }
    }
}

// Payload untuk membuat jadwal kuliah baru
#[derive(Debug, Deserialize)]
pub struct CreateJadwalKuliahPayload {
    pub matakuliah_id: Uuid,
    pub tahun_akademik_id: Uuid,
    pub hari: DayOfWeek,
    #[serde(with = "time_format_hm")]
    pub jam_mulai: Time,
    #[serde(with = "time_format_hm")]
    pub jam_selesai: Time,
    pub kelas: String,
    pub dosen_pengampu: Vec<DosenPengampuPayload>, // Daftar dosen
}


#[derive(Debug, Deserialize)]
pub struct PlotJadwalRuanganPayload {
    pub jadwal_kuliah_id: Uuid,
    pub ruangan_id: Uuid,
}


#[derive(Debug, Serialize)]
pub struct DosenPengampuDetail {
    pub dosen_id: Uuid,
    pub nama_dosen: String,
    pub peran: PeranDosenPengampu,
}

#[derive(Debug, Serialize)]
pub struct JadwalKuliahDetail {
    pub id: Uuid,
    pub kelas: String,
    pub hari: DayOfWeek,
    pub jam_mulai: Time,
    pub jam_selesai: Time,
    pub matakuliah_id: Uuid,
    pub nama_mk: String,
    pub kode_mk: String,
    pub sks: i32,
    pub prodi_id: Uuid,
    pub nama_prodi: String,
    pub tahun_akademik_id: Uuid,
    pub nama_tahun_akademik: String,
    pub dosen_pengampu: Vec<DosenPengampuDetail>,
}


#[derive(Debug, Deserialize)]
pub struct JadwalKuliahFilter {
    pub tahun_akademik_id: Option<Uuid>,
    pub prodi_id: Option<Uuid>,
}
#[derive(Debug, Deserialize)]
pub struct UpdateJadwalKuliahPayload {
    pub matakuliah_id: Uuid,
    pub tahun_akademik_id: Uuid,
    pub hari: DayOfWeek,
    #[serde(with = "time_format_hm")]
    pub jam_mulai: Time,
    #[serde(with = "time_format_hm")]
    pub jam_selesai: Time,    
    pub kelas: String,
    pub dosen_pengampu: Vec<DosenPengampuPayload>,
}

