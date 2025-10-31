// src/modules/sdm/karir_dosen_model.rs
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use time::{Date};
use uuid::Uuid;

// --- ENUM UNTUK JABATAN AKADEMIK (JAD) ---

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "JabatanAkademik")]
pub enum JabatanAkademik {
    #[serde(rename = "Asisten Ahli")]
    #[sqlx(rename = "Asisten Ahli")]
    AsistenAhli,
    Lektor,
    #[serde(rename = "Lektor Kepala")]
    #[sqlx(rename = "Lektor Kepala")]
    LektorKepala,
    #[serde(rename = "Guru Besar")]
    #[sqlx(rename = "Guru Besar")]
    GuruBesar,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "PangkatGolongan")]
pub enum PangkatGolongan {
    #[serde(rename = "III/a")]
    #[sqlx(rename = "III/a")]
    IIIa,
    #[serde(rename = "III/b")]
    #[sqlx(rename = "III/b")]
    IIIb,
    #[serde(rename = "III/c")]
    #[sqlx(rename = "III/c")]
    IIIc,
    #[serde(rename = "III/d")]
    #[sqlx(rename = "III/d")]
    IIId,
    #[serde(rename = "IV/a")]
    #[sqlx(rename = "IV/a")]
    IVa,
    #[serde(rename = "IV/b")]
    #[sqlx(rename = "IV/b")]
    IVb,
    #[serde(rename = "IV/c")]
    #[sqlx(rename = "IV/c")]
    IVc,
    #[serde(rename = "IV/d")]
    #[sqlx(rename = "IV/d")]
    IVd,
    #[serde(rename = "IV/e")]
    #[sqlx(rename = "IV/e")]
    IVe,
}

// --- Helper as_str() ---
impl JabatanAkademik {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AsistenAhli => "Asisten Ahli",
            Self::Lektor => "Lektor",
            Self::LektorKepala => "Lektor Kepala",
            Self::GuruBesar => "Guru Besar",
        }
    }
}
impl PangkatGolongan {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IIIa => "III/a",
            Self::IIIb => "III/b",
            Self::IIIc => "III/c",
            Self::IIId => "III/d",
            Self::IVa => "IV/a",
            Self::IVb => "IV/b",
            Self::IVc => "IV/c",
            Self::IVd => "IV/d",
            Self::IVe => "IV/e",
        }
    }
}

// --- STRUCT UNTUK JAD ---

#[derive(Debug, Serialize, FromRow)]
pub struct RiwayatJad {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    pub jabatan_akademik: JabatanAkademik,
    pub pangkat_golongan: PangkatGolongan,
    pub nomor_sk: String,
    pub tmt: Date,
    pub kompetensi_mk: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RiwayatJadPayload {
    pub jabatan_akademik: JabatanAkademik,
    pub pangkat_golongan: PangkatGolongan,
    pub nomor_sk: String,
    pub tmt: Date,
    pub kompetensi_mk: Option<String>,
}

// --- STRUCT UNTUK SERDOS ---

#[derive(Debug, Serialize, FromRow)]
pub struct RiwayatSerdos {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    pub nomor_sertifikat: String,
    pub tanggal_terbit: Date,
    pub keterangan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RiwayatSerdosPayload {
    pub nomor_sertifikat: String,
    pub tanggal_terbit: Date,
    pub keterangan: Option<String>,
}