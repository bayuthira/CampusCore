// src/modules/sdm/surat_tugas_model.rs
use serde::{Deserialize, Serialize};
use sqlx::Type;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

// --- ENUM BARU DARI MIGRasi ---
#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "PeranPerjalanan")]
pub enum PeranPerjalanan {
    #[serde(rename = "Pelaksana Utama")]
    #[sqlx(rename = "Pelaksana Utama")]
    PelaksanaUtama,
    Pengikut,
}

impl PeranPerjalanan {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PelaksanaUtama => "Pelaksana Utama",
            Self::Pengikut => "Pengikut",
        }
    }
}

// --- STRUCT UNTUK DATABASE (Raw) ---
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SuratTugas {
    pub id: Uuid,
    pub nomor_surat: Option<String>, // <-- PERBAIKAN: Menjadi Option
    pub dasar_tugas: Option<String>,
    pub tugas: String,
    pub tempat_tugas: String,
    pub tanggal_mulai: Date,
    pub tanggal_selesai: Date,
    pub penandatangan_id: Uuid,
    pub tembusan: Option<Vec<String>>,
    pub user_pembuat_id: Uuid,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,

    // Kolom SPPD (Opsional)
    pub nomor_sppd: Option<String>,
    pub alasan_perjalanan: Option<i32>, // 1: Kunjungan/Undangan, 2: Tugas Lembaga, 3: Pelatihan
    pub tujuan_kota: Option<String>,
    pub alat_angkut: Option<String>,
    pub tempat_berangkat: Option<String>,
    pub lama_perjalanan: Option<i32>,
    pub pembebanan_anggaran_instansi: Option<String>,
    pub pembebanan_anggaran_mak: Option<String>,
    pub ppk_pegawai_id: Option<Uuid>,
    pub kpa_pegawai_id: Option<Uuid>,
    pub keterangan_lain: Option<String>,
}

// --- STRUCT UNTUK RESPONS API (Detail) ---

#[derive(Debug, Serialize)]
pub struct PenerimaTugasDetail {
    pub pegawai_id: Uuid,
    pub nama_lengkap: String,
    pub nip: String,
    pub jabatan: Option<String>,
    pub unit_kerja: Option<String>, // <-- TAMBAHAN (Pastikan ini ada)
    pub pangkat_golongan: Option<String>,
    pub peran: PeranPerjalanan,
}

#[derive(Debug, Serialize)]
pub struct SuratTugasDetail {
    pub id: Uuid,
    pub nomor_surat: Option<String>, // <-- PERBAIKAN: Menjadi Option
    pub dasar_tugas: Option<String>,
    pub tugas: String,
    pub tempat_tugas: String,
    pub tanggal_mulai: Date,
    pub tanggal_selesai: Date,

    // Info Penandatangan
    pub penandatangan_id: Uuid,
    pub nama_penandatangan: String,
    pub jabatan_penandatangan: Option<String>,
    pub nip_penandatangan: String,

    // Info Penerima (Array)
    pub daftar_penerima: Vec<PenerimaTugasDetail>,

    pub tembusan: Vec<String>,
    pub created_at: OffsetDateTime,

    // Kolom SPPD (Opsional)
    pub nomor_sppd: Option<String>,
    pub alasan_perjalanan: Option<i32>,
    pub tujuan_kota: Option<String>,
    pub alat_angkut: Option<String>,
    pub tempat_berangkat: Option<String>,
    pub lama_perjalanan: Option<i32>,
    pub pembebanan_anggaran_instansi: Option<String>,
    pub pembebanan_anggaran_mak: Option<String>,

    // Info Pejabat SPPD (Opsional)
    pub ppk_pegawai_id: Option<Uuid>,
    pub nama_ppk: Option<String>,
    pub kpa_pegawai_id: Option<Uuid>,
    pub nama_kpa: Option<String>,

    pub keterangan_lain: Option<String>,
}

// --- STRUCT UNTUK PAYLOAD (Create/Update) ---
#[derive(Debug, Deserialize)]
pub struct PenerimaTugasPayload {
    pub pegawai_id: Uuid,
    pub peran: PeranPerjalanan,
}

#[derive(Debug, Deserialize)]
pub struct CreateSuratTugasPayload {
    pub dasar_tugas: Option<String>,
    pub tugas: String,
    pub tempat_tugas: String,
    pub tanggal_mulai: Date,
    pub tanggal_selesai: Date,
    pub penandatangan_id: Uuid,
    pub tembusan: Option<Vec<String>>,
    pub penerima_tugas: Vec<PenerimaTugasPayload>,

    // Kolom Tambahan SPPD
    pub alasan_perjalanan: Option<i32>,
    pub tujuan_kota: Option<String>,
    pub alat_angkut: Option<String>,
    pub tempat_berangkat: Option<String>,
    pub lama_perjalanan: Option<i32>,
    pub pembebanan_anggaran_instansi: Option<String>,
    pub pembebanan_anggaran_mak: Option<String>,
    pub ppk_pegawai_id: Option<Uuid>,
    pub kpa_pegawai_id: Option<Uuid>,
    pub keterangan_lain: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSuratTugasPayload {
    pub dasar_tugas: Option<String>,
    pub tugas: Option<String>,
    pub tempat_tugas: Option<String>,
    #[serde(default)]
    pub tanggal_mulai: Option<Date>,
    #[serde(default)]
    pub tanggal_selesai: Option<Date>,
    pub penandatangan_id: Option<Uuid>,
    pub tembusan: Option<Vec<String>>,
    pub penerima_tugas: Option<Vec<PenerimaTugasPayload>>,

    // Kolom Tambahan SPPD
    pub alasan_perjalanan: Option<i32>,
    pub tujuan_kota: Option<String>,
    pub alat_angkut: Option<String>,
    pub tempat_berangkat: Option<String>,
    pub lama_perjalanan: Option<i32>,
    pub pembebanan_anggaran_instansi: Option<String>,
    pub pembebanan_anggaran_mak: Option<String>,
    pub ppk_pegawai_id: Option<Uuid>,
    pub kpa_pegawai_id: Option<Uuid>,
    pub keterangan_lain: Option<String>,
}
