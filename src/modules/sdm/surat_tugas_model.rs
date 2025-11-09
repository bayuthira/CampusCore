// src/modules/sdm/surat_tugas_model.rs
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

// --- STRUCT UNTUK DATABASE (Raw) ---
// Merepresentasikan satu baris di tabel surat_tugas_master
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SuratTugas {
    pub id: Uuid,
    pub nomor_surat: String,
    pub dasar_tugas: Option<String>,
    pub tugas: String,
    pub tempat_tugas: String,
    pub tanggal_mulai: Date,
    pub tanggal_selesai: Date,
    pub penandatangan_id: Uuid,
    pub tembusan: Option<Vec<String>>, // PostgreSQL TEXT[] dipetakan ke Vec<String>
    pub user_pembuat_id: Uuid,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// --- STRUCT UNTUK RESPONS API (Detail) ---
// Ini yang akan dikirim ke Frontend, sudah termasuk nama-nama pegawai.

#[derive(Debug, Serialize)]
pub struct PenerimaTugasDetail {
    pub pegawai_id: Uuid,
    pub nama_lengkap: String,
    pub nip: String,
    pub jabatan: Option<String>,          // Jabatan saat surat dibuat
    pub pangkat_golongan: Option<String>, // Pangkat saat surat dibuat (jika ada)
}

#[derive(Debug, Serialize)]
pub struct SuratTugasDetail {
    pub id: Uuid,
    pub nomor_surat: String,
    pub dasar_tugas: Option<String>,
    pub tugas: String,
    pub tempat_tugas: String,
    pub tanggal_mulai: Date,
    pub tanggal_selesai: Date,

    // Info Penandatangan
    pub penandatangan_id: Uuid,
    pub nama_penandatangan: String,
    pub jabatan_penandatangan: Option<String>, // Jabatan struktural penandatangan saat ini
    pub nip_penandatangan: String,

    // Info Penerima (Array)
    pub daftar_penerima: Vec<PenerimaTugasDetail>,

    pub tembusan: Vec<String>,
    pub created_at: OffsetDateTime,
}

// --- STRUCT UNTUK PAYLOAD (Create/Update) ---

#[derive(Debug, Deserialize)]
pub struct CreateSuratTugasPayload {
    pub dasar_tugas: Option<String>,
    pub tugas: String,
    pub tempat_tugas: String,
    pub tanggal_mulai: Date,
    pub tanggal_selesai: Date,

    pub penandatangan_id: Uuid,        // Siapa yang menandatangani
    pub penerima_tugas_ids: Vec<Uuid>, // Daftar ID pegawai yang ditugaskan

    pub tembusan: Option<Vec<String>>, // Contoh: ["Arsip", "Ketua Prodi"]
}

#[derive(Debug, Deserialize)]
pub struct UpdateSuratTugasPayload {
    // Hampir semua bisa diubah KECUALI nomor surat (idealnya)
    pub dasar_tugas: Option<String>,
    pub tugas: Option<String>,
    pub tempat_tugas: Option<String>,
    #[serde(default)]
    pub tanggal_mulai: Option<Date>,
    #[serde(default)]
    pub tanggal_selesai: Option<Date>,

    pub penandatangan_id: Option<Uuid>,
    pub penerima_tugas_ids: Option<Vec<Uuid>>, // Jika diisi, akan me-replace daftar lama

    pub tembusan: Option<Vec<String>>,
}
