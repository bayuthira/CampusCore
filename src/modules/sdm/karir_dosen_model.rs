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
    #[serde(rename = "Penata Muda / III.a")]
    #[sqlx(rename = "Penata Muda / III.a")]
    PenataMuda,

    #[serde(rename = "Penata Muda Tk.I / III.b")]
    #[sqlx(rename = "Penata Muda Tk.I / III.b")]
    PenataMudaTkI,

    #[serde(rename = "Penata / III.c")]
    #[sqlx(rename = "Penata / III.c")]
    Penata,

    #[serde(rename = "Penata Tk. I / III.d")]
    #[sqlx(rename = "Penata Tk. I / III.d")]
    PenataTkI,

    #[serde(rename = "Pembina / IV.a")]
    #[sqlx(rename = "Pembina / IV.a")]
    Pembina,

    #[serde(rename = "Pembina Tk. I / IV.b")]
    #[sqlx(rename = "Pembina Tk. I / IV.b")]
    PembinaTkI,

    #[serde(rename = "Pembina Utama Muda / IV.c")]
    #[sqlx(rename = "Pembina Utama Muda / IV.c")]
    PembinaUtamaMuda,

    #[serde(rename = "Pembina Utama Madya / IV.d")]
    #[sqlx(rename = "Pembina Utama Madya / IV.d")]
    PembinaUtamaMadya,
    
    #[serde(rename = "Pembina Utama / IV.e")]
    #[sqlx(rename = "Pembina Utama / IV.e")]
    PembinaUtama,
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
            Self::PenataMuda => "Penata Muda / III.a",
            Self::PenataMudaTkI => "Penata Muda Tk.I / III.b",
            Self::Penata => "Penata / III.c",
            Self::PenataTkI => "Penata Tk. I / III.d",
            Self::Pembina => "Pembina / IV.a",
            Self::PembinaTkI => "Pembina Tk. I / IV.b",
            Self::PembinaUtamaMuda => "Pembina Utama Muda / IV.c",
            Self::PembinaUtamaMadya => "Pembina Utama Madya / IV.d",
            Self::PembinaUtama => "Pembina Utama / IV.e",
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