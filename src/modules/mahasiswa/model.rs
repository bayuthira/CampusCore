// src/modules/mahasiswa/model.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Payload untuk membuat Mahasiswa baru
#[derive(Debug, Deserialize)]
pub struct CreateMahasiswaPayload {
    // --- BIODATA (Tabel mahasiswa) ---
    pub nik: String,
    pub nama_mahasiswa: String,
    pub email: Option<String>,
    pub tempat_lahir: Option<String>,
    pub tanggal_lahir: Option<time::Date>,
    pub nama_ibu_kandung: Option<String>,

    // --- AKADEMIK (Tabel registrasi_mahasiswa) ---
    pub nim: String,
    pub angkatan: i32,
    pub prodi_id: Uuid,
    pub periode_masuk: Option<String>, // Contoh: "20241"

    // --- AUTH ---
    pub password: String, // Password awal untuk akun login mahasiswa
}

// Struct untuk menampilkan detail Mahasiswa, hasil dari JOIN beberapa tabel.
#[derive(Debug, Serialize, FromRow)]
pub struct MahasiswaDetail {
    pub id: Uuid,                    // id dari tabel mahasiswa (Biodata)
    pub registrasi_id: Option<Uuid>, // id dari tabel registrasi_mahasiswa (Sangat penting untuk KRS)

    // Biodata
    pub nik: Option<String>,
    pub nama_mahasiswa: String,
    pub email: Option<String>,

    // Akademik
    pub nim: Option<String>,
    pub angkatan: Option<i32>,
    pub prodi_id: Option<Uuid>,
    pub nama_prodi: String,
    pub status_mahasiswa: Option<String>,

    // DOSEN PA
    pub dosen_pa_id: Option<Uuid>,
    pub nama_dosen_pa: Option<String>,

    // Akun
    pub user_id: Option<Uuid>,
    pub username: String, // username dari tabel users
}

// Skema untuk Impor CSV (Ditambah NIK)
#[derive(Debug, Deserialize)]
pub struct MahasiswaCsvRecord {
    pub nik: String,
    pub nim: String,
    pub nama_mahasiswa: String,
    pub email: String,
    pub angkatan: i32,
    pub kode_prodi: String,
}

// Struct untuk laporan hasil impor
#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub status: String, // Contoh: "SUKSES" atau "GAGAL_DIBATALKAN"
    pub total_baris_dipindai: u32,
    pub baris_berhasil_disimpan: u32,
    pub detail_error: Vec<String>,
}

// Menggunakan Option<> untuk Partial Update (Update sebagian)
#[derive(Debug, Deserialize)]
pub struct UpdateMahasiswaPayload {
    // Biodata
    pub nik: Option<String>,
    pub nama_mahasiswa: Option<String>,
    pub email: Option<String>,
    pub tempat_lahir: Option<String>,
    pub tanggal_lahir: Option<time::Date>,
    pub nama_ibu_kandung: Option<String>,

    // Akademik
    pub nim: Option<String>,
    pub angkatan: Option<i32>,
    pub prodi_id: Option<Uuid>,
    pub status_mahasiswa: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MahasiswaBimbingan {
    pub id: Uuid,
    pub nim: String,
    pub nama_mahasiswa: String,
    pub angkatan: i32,
    pub email: Option<String>,
    pub nama_prodi: String,
}

// --- PAYLOAD UNTUK ASSIGN DOSEN PA ---

#[derive(Debug, Deserialize)]
pub struct BatchAssignDosenPaPayload {
    pub prodi_id: Uuid,
    pub angkatan: i32,
    pub kode_rombel: String,
    pub dosen_pa_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct SingleAssignDosenPaPayload {
    pub registrasi_id: Uuid,
    pub dosen_pa_id: Uuid,
}

// =========================================================================
// --- MODEL UNTUK FITUR MANAJEMEN ROMBEL ---
// =========================================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RombelSummary {
    pub prodi_id: Uuid,
    pub nama_prodi: String,
    pub angkatan: i32,
    pub kode_rombel: Option<String>,
    pub jumlah_mahasiswa: Option<i64>,
    pub dosen_pa_id: Option<Uuid>,
    pub nama_dosen_pa: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RombelFilter {
    pub prodi_id: Option<Uuid>,
    pub angkatan: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct MahasiswaRombelFilter {
    pub prodi_id: Uuid,
    pub angkatan: i32,
    pub kode_rombel: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MahasiswaRombelDetail {
    pub registrasi_id: Uuid,
    pub mahasiswa_id: Uuid,
    pub nim: String,
    pub nama_mahasiswa: String,
    pub kode_rombel: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PindahRombelPayload {
    pub registrasi_ids: Vec<Uuid>,
    pub kode_rombel_baru: Option<String>, // Bisa diset null jika ingin dicabut rombelnya
}

#[derive(Debug, Deserialize)]
pub struct RenameRombelPayload {
    pub prodi_id: Uuid,
    pub angkatan: i32,
    pub kode_rombel_lama: Option<String>,
    pub kode_rombel_baru: Option<String>,
}
