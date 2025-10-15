// src/modules/sdm/model.rs
use serde::{Deserialize, Serialize};
use sqlx::{Type,FromRow};
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
#[sqlx(type_name = "KategoriPegawai")] // Nama tipe di DB
pub enum KategoriPegawai {
    #[serde(rename = "Tenaga Pendidik")] // Untuk JSON (serde)
    #[sqlx(rename = "Tenaga Pendidik")]  // Untuk Database (sqlx)
    TenagaPendidik,

    #[serde(rename = "Tenaga Kependidikan")] // Untuk JSON (serde)
    #[sqlx(rename = "Tenaga Kependidikan")]  // Untuk Database (sqlx)
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
    pub unit_kerja: Option<String>,
    pub bagian: Option<String>,
    pub jabatan: Option<String>,
    pub tanggal_masuk: Option<Date>,
    pub tanggal_pensiun: Option<Date>,
    pub no_kk: Option<String>,
    pub no_npwp: Option<String>,
    pub no_bpjs_kesehatan: Option<String>,
    pub no_bpjs_ketenagakerjaan: Option<String>,
    // Data Dosen (jika ada)
    pub nidn: Option<String>,
    pub prodi_id: Option<Uuid>,
    pub nama_prodi: Option<String>,
    pub created_at: OffsetDateTime,
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
    pub unit_kerja: Option<String>,
    pub bagian: Option<String>,
    pub jabatan: Option<String>,
    pub tanggal_masuk: Option<Date>,
    pub tanggal_pensiun: Option<Date>,
    pub no_kk: Option<String>,
    pub no_npwp: Option<String>,
    pub no_bpjs_kesehatan: Option<String>,
    pub no_bpjs_ketenagakerjaan: Option<String>,
    pub nidn: Option<String>, 
    pub prodi_id: Option<Uuid>, 
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
