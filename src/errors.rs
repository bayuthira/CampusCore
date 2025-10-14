use axum::extract::multipart::MultipartError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rust_decimal::Error as DecimalError;
use serde_json::Error as SerdeJsonError;
use serde_json::json;
use std::io::Error as IoError;
use time::error::Parse as TimeParseError;
use tracing;
use uuid::Error as UuidError;

#[derive(Debug)]
pub enum AppError {
    SqlxError(sqlx::Error),
    #[allow(dead_code)] // Izinkan "dead code" karena hanya dipakai di log
    BcryptError(bcrypt::BcryptError),
    #[allow(dead_code)] // Izinkan "dead code" karena hanya dipakai di log
    JsonWebTokenError(jsonwebtoken::errors::Error),
    AnyhowError(anyhow::Error),
    Forbidden(String),
    MultipartError(MultipartError),
    DuplicateEntry(String),
    IoError(IoError),
    UuidError(UuidError),
    SerdeJsonError(SerdeJsonError),
    TimeParseError(TimeParseError),
    DecimalError(DecimalError),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::SqlxError(err)
    }
}
impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::BcryptError(err)
    }
}
impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::JsonWebTokenError(err)
    }
}
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::AnyhowError(err)
    }
}
impl From<MultipartError> for AppError {
    fn from(err: MultipartError) -> Self {
        AppError::MultipartError(err)
    }
}
impl From<DecimalError> for AppError {
    fn from(err: DecimalError) -> Self {
        AppError::DecimalError(err)
    }
}

// --- TAMBAHKAN DUA IMPLEMENTASI `From` BARU DI SINI ---
impl From<IoError> for AppError {
    fn from(err: IoError) -> Self {
        AppError::IoError(err)
    }
}

impl From<UuidError> for AppError {
    fn from(err: UuidError) -> Self {
        AppError::UuidError(err)
    }
}

impl From<SerdeJsonError> for AppError {
    fn from(err: SerdeJsonError) -> Self {
        AppError::SerdeJsonError(err)
    }
}

impl From<TimeParseError> for AppError {
    fn from(err: TimeParseError) -> Self {
        AppError::TimeParseError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log error asli ke konsol server
        tracing::error!("--> An error occurred: {:#?}", &self);

        let (status, error_message) = match self {
            AppError::SqlxError(err) => {
                // Ubah error sqlx menjadi string untuk dianalisis
                let err_string = err.to_string();

                // Cek apakah ini adalah error unique constraint
                if err_string.contains("violates unique constraint") {
                    println!("--> DEBUG: Raw unique constraint error: {}", err_string);
                    // Jika ya, cari tahu constraint mana yang dilanggar
                    let message = if err_string.contains("users_username_key") {
                        "Username ini sudah terdaftar.".to_string()
                    } else if err_string.contains("users_email_key")
                        || err_string.contains("mahasiswa_email_key")
                    {
                        "Email ini sudah terdaftar.".to_string()
                    } else if err_string.contains("dosen_nidn_key") {
                        "NIDN ini sudah terdaftar.".to_string()
                    } else if err_string.contains("mahasiswa_nim_key") {
                        "NIM ini sudah terdaftar.".to_string()
                    } else if err_string.contains("prodi_kode_prodi_key") {
                        "Kode Prodi ini sudah ada.".to_string()
                    } else if err_string.contains("mata_kuliah_kode_mk_key") {
                        "Kode Mata Kuliah ini sudah digunakan.".to_string()
                    } else if err_string.contains("tahun_akademik_nama_key") {
                        "Nama Tahun Akademik ini sudah digunakan.".to_string()
                    } else if err_string
                        .contains("enrollments_mahasiswa_id_matakuliah_id_tahun_akademik_id_key")
                    {
                        "Anda sudah mengambil mata kuliah ini di periode yang sama.".to_string()
                    } else if err_string.contains("kurikulum_prodi_id_nama_key") {
                        "Nama kurikulum untuk prodi ini sudah ada.".to_string()
                    } else if err_string.contains("ruangan_kode_ruangan_key") {
                        "Kode Ruangan ini sudah digunakan.".to_string()
                    } else if err_string.contains("jenis_aset_nama_jenis_key") {
                        "Nama Jenis Aset ini sudah ada.".to_string()
                    } else if err_string.contains("aset_kode_aset_key") {
                        "Kode Aset ini sudah digunakan.".to_string()
                    } else if err_string.contains("aset_habis_pakai_nama_barang_key") {
                        "Nama barang ini sudah terdaftar.".to_string()
                    } else if err_string
                        .contains("jadwal_kuliah_matakuliah_id_tahun_akademik_id_kelas_key")
                    {
                        "Jadwal untuk mata kuliah ini di kelas dan tahun akademik yang sama sudah ada.".to_string()
                    } else if err_string.contains("kendaraan_nomor_polisi_key") {
                        "Nomor Polisi ini sudah terdaftar.".to_string()
                    } else if err_string.contains("pegawai_nik_key") {
                        "NIK Pegawai ini sudah terdaftar.".to_string()
                    } else if err_string.contains("pegawai_no_ktp_key") {
                        "No KTP ini sudah terdaftar.".to_string()
                    } else if err_string.contains("pegawai_email_key") {
                        "Email ini sudah terdaftar untuk pegawai lain.".to_string()
                    } else if err_string.contains("pegawai_user_id_key") {
                        "Akun user dengan email ini sudah terhubung dengan data pegawai lain."
                            .to_string()
                    } else {
                        // Pesan fallback jika constraint tidak dikenali
                        "Data yang Anda masukkan sudah ada di sistem (nilai duplikat).".to_string()
                    };

                    // Kembalikan 409 Conflict dengan pesan yang sudah diterjemahkan
                    (StatusCode::CONFLICT, message)
                } else if err_string.contains("violates foreign key constraint") {
                    // <-- TAMBAHKAN BLOK INI
                    // Kode untuk 23503 (foreign key)
                    // --- TAMBAHKAN BARIS INI UNTUK DEBUGGING ---
                    println!("--> DEBUG Foreign Key Error: {}", err_string);

                    let message = if err_string.contains("dosen_user_id_fkey")
                        || err_string.contains("mahasiswa_user_id_fkey")
                    {
                        "User tidak dapat dihapus karena masih terdaftar sebagai Dosen atau Mahasiswa.".to_string()
                    } else {
                        "Operasi gagal karena melanggar relasi data.".to_string()
                    };
                    (StatusCode::CONFLICT, message) // 409 Conflict cocok untuk kasus ini
                } else {
                    // --- PERUBAHAN DI SINI ---
                    // Tampilkan error asli saat development, pesan generik saat release
                    let body = {
                        #[cfg(debug_assertions)]
                        {
                            // Mode Debug (`cargo run`)
                            err.to_string()
                        }
                        #[cfg(not(debug_assertions))]
                        {
                            // Mode Release (`cargo run --release`)
                            "Terjadi masalah pada database.".to_string()
                        }
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, body)
                }
            }
            AppError::DuplicateEntry(message) => (StatusCode::CONFLICT, message.clone()),
            AppError::BcryptError(_) | AppError::JsonWebTokenError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Terjadi kesalahan pemrosesan internal.".to_string(),
            ),
            AppError::AnyhowError(err) => (StatusCode::UNAUTHORIZED, err.to_string()),
            AppError::Forbidden(message) => (StatusCode::FORBIDDEN, message.clone()),
            AppError::MultipartError(err) => (
                StatusCode::BAD_REQUEST,
                format!("Request upload tidak valid: {}", err),
            ),
            AppError::IoError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Terjadi masalah I/O file: {}", err),
            ),
            AppError::UuidError(err) => (
                StatusCode::BAD_REQUEST,
                format!("Format ID tidak valid: {}", err),
            ),
            AppError::SerdeJsonError(err) => (
                StatusCode::BAD_REQUEST,
                format!("Format JSON tidak valid: {}", err),
            ),
            AppError::TimeParseError(err) => (
                StatusCode::BAD_REQUEST,
                format!("Format tanggal tidak valid: {}", err),
            ),
            AppError::DecimalError(err) => (
                StatusCode::BAD_REQUEST,
                format!("Format angka tidak valid: {}", err),
            ),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
