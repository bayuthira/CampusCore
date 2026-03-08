// src/modules/sdm/model.rs
use super::dokumen_model::DokumenSdmSimple;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use time::{Date, OffsetDateTime};
use uuid::Uuid;
// --- ENUM untuk tipe data kustom ---

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "JenisKelamin")]
pub enum JenisKelamin {
    L,
    P,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "StatusNikah")]
pub enum StatusNikah {
    Menikah,
    #[serde(rename = "Belum Menikah")]
    BelumMenikah,
    #[serde(rename = "Cerai Hidup")]
    CeraiHidup,
    #[serde(rename = "Cerai Mati")]
    CeraiMati,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "KategoriPegawai")]
pub enum KategoriPegawai {
    #[serde(rename = "Tenaga Pendidik")]
    #[sqlx(rename = "Tenaga Pendidik")]
    TenagaPendidik,

    #[serde(rename = "Tenaga Kependidikan")]
    #[sqlx(rename = "Tenaga Kependidikan")]
    TenagaKependidikan,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "StatusPegawai")]
pub enum StatusPegawai {
    Tetap,
    Kontrak,
    Honorer,
}

// --- Struct untuk Respons API ---
#[derive(Debug, Serialize, FromRow)]
pub struct Pegawai {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub nik: String,
    pub no_ktp: Option<String>,
    pub nama_lengkap: String,
    pub gelar_depan: Option<String>,
    pub gelar_belakang: Option<String>,
    pub tempat_lahir: Option<String>,
    pub tanggal_lahir: Option<Date>,
    pub jenis_kelamin: Option<JenisKelamin>,
    pub status_nikah: Option<StatusNikah>,
    pub agama: Option<String>,
    pub gol_darah: Option<String>,
    pub alamat_domisili: Option<String>,
    pub kota: Option<String>,
    pub kode_pos: Option<String>,
    pub nomor_hp: Option<String>,
    pub email: Option<String>,
    pub kategori_pegawai: Option<KategoriPegawai>,
    pub status_pegawai: Option<StatusPegawai>,
    pub is_active: bool,
    pub tanggal_masuk: Option<Date>,
    pub tanggal_pensiun: Option<Date>,
    pub no_kk: Option<String>,
    pub no_npwp: Option<String>,
    pub no_bpjs_kesehatan: Option<String>,
    pub no_bpjs_ketenagakerjaan: Option<String>,

    // TAMBAHAN FEEDER PEGAWAI
    pub id_sdm_feeder: Option<Uuid>,
    pub nama_ibu_kandung: Option<String>,
    pub kewarganegaraan: Option<String>,
    pub dusun: Option<String>,
    pub rt: Option<String>,
    pub rw: Option<String>,
    pub kelurahan: Option<String>,
    pub id_wilayah_feeder: Option<Uuid>,

    // Data Dosen (jika ada)
    pub nidn: Option<String>,
    pub prodi_id: Option<Uuid>,
    pub nama_prodi: Option<String>,
    pub id_penugasan_feeder: Option<Uuid>,
    pub ikatan_kerja: Option<String>,

    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

// --- Struct untuk Payload (Create & Update) ---
#[derive(Debug, Deserialize)]
pub struct PegawaiPayload {
    pub nik: String,
    pub no_ktp: Option<String>,
    pub nama_lengkap: String,
    pub gelar_depan: Option<String>,
    pub gelar_belakang: Option<String>,
    pub tempat_lahir: Option<String>,
    pub tanggal_lahir: Option<Date>,
    pub jenis_kelamin: Option<JenisKelamin>,
    pub status_nikah: Option<StatusNikah>,
    pub agama: Option<String>,
    pub gol_darah: Option<String>,
    pub alamat_domisili: Option<String>,
    pub kota: Option<String>,
    pub kode_pos: Option<String>,
    pub nomor_hp: Option<String>,
    pub email: Option<String>,
    pub kategori_pegawai: Option<KategoriPegawai>,
    pub status_pegawai: Option<StatusPegawai>,
    pub is_active: Option<bool>,
    pub unit_kerja_id: Option<Uuid>,
    pub jabatan: Option<String>,
    pub tanggal_masuk: Option<Date>,
    pub tanggal_pensiun: Option<Date>,
    pub no_kk: Option<String>,
    pub no_npwp: Option<String>,
    pub no_bpjs_kesehatan: Option<String>,
    pub no_bpjs_ketenagakerjaan: Option<String>,

    // TAMBAHAN FEEDER PEGAWAI
    pub id_sdm_feeder: Option<Uuid>,
    pub nama_ibu_kandung: Option<String>,
    pub kewarganegaraan: Option<String>,
    pub dusun: Option<String>,
    pub rt: Option<String>,
    pub rw: Option<String>,
    pub kelurahan: Option<String>,
    pub id_wilayah_feeder: Option<Uuid>,

    // Data Dosen (jika Tenaga Pendidik)
    pub nidn: Option<String>,
    pub prodi_id: Option<Uuid>,
    pub id_penugasan_feeder: Option<Uuid>,
    pub ikatan_kerja: Option<String>,

    // Auth
    pub password: Option<String>,
}

impl JenisKelamin {
    pub fn as_str(&self) -> &'static str {
        match self {
            JenisKelamin::L => "L",
            JenisKelamin::P => "P",
        }
    }
}

impl StatusNikah {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatusNikah::Menikah => "Menikah",
            StatusNikah::BelumMenikah => "Belum Menikah",
            StatusNikah::CeraiHidup => "Cerai Hidup",
            StatusNikah::CeraiMati => "Cerai Mati",
        }
    }
}

impl KategoriPegawai {
    pub fn as_str(&self) -> &'static str {
        match self {
            KategoriPegawai::TenagaPendidik => "Tenaga Pendidik",
            KategoriPegawai::TenagaKependidikan => "Tenaga Kependidikan",
        }
    }
}

impl StatusPegawai {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatusPegawai::Tetap => "Tetap",
            StatusPegawai::Kontrak => "Kontrak",
            StatusPegawai::Honorer => "Honorer",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateUserForPegawaiPayload {
    pub password: String,
}

// --- Struct untuk Riwayat Pendidikan ---

#[derive(Debug, sqlx::FromRow)]
pub struct RiwayatPendidikanDb {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    pub jenjang: String,
    pub institusi: String,
    pub jurusan: Option<String>,
    pub tahun_lulus: Option<i16>,
}

#[derive(Debug, Serialize)]
pub struct RiwayatPendidikanDetail {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    pub jenjang: String,
    pub institusi: String,
    pub jurusan: Option<String>,
    pub tahun_lulus: Option<i16>,
    pub dokumen: Vec<DokumenSdmSimple>,
}

#[derive(Debug, Deserialize)]
pub struct RiwayatPendidikanPayload {
    pub jenjang: String,
    pub institusi: String,
    pub jurusan: Option<String>,
    pub tahun_lulus: Option<i16>,
}

// --- Struct untuk Riwayat SK ---

#[derive(Debug, sqlx::FromRow)]
pub struct RiwayatSkDb {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    pub nomor_sk: String,
    pub tanggal_sk: Date,
    pub jenis_sk: String,
    pub jabatan: Option<String>,
    pub keterangan: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RiwayatSkDetail {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    pub nomor_sk: String,
    pub tanggal_sk: Date,
    pub jenis_sk: String,
    pub jabatan: Option<String>,
    pub keterangan: Option<String>,
    pub dokumen: Vec<DokumenSdmSimple>,
}

#[derive(Debug, Deserialize)]
pub struct RiwayatSkPayload {
    pub nomor_sk: String,
    pub tanggal_sk: Date,
    pub jenis_sk: String,
    pub jabatan: Option<String>,
    pub keterangan: Option<String>,
}

// --- Struct untuk Riwayat Penempatan ---

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PenempatanPegawai {
    pub id: Uuid,
    pub pegawai_id: Uuid,
    pub unit_kerja_id: Uuid,
    pub nama_unit_kerja: String,
    pub jabatan: String,
    pub nomor_sk: Option<String>,
    pub tanggal_mulai: Date,
    pub tanggal_selesai: Option<Date>,
}

#[derive(Debug, Deserialize)]
pub struct PenempatanPegawaiPayload {
    pub unit_kerja_id: Uuid,
    pub jabatan: String,
    pub nomor_sk: Option<String>,
    pub tanggal_mulai: Date,
}
