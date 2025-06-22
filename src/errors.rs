// src/errors.rs

use axum::extract::multipart::MultipartError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

// --- MODIFIKASI 1: Menambahkan varian error baru ---
#[derive(Debug)]
pub enum AppError {
    SqlxError(sqlx::Error),
    BcryptError(bcrypt::BcryptError),
    JsonWebTokenError(jsonwebtoken::errors::Error),
    AnyhowError(anyhow::Error),
    Forbidden,
    MultipartError(MultipartError),
    DuplicateEntry(String),
}

// --- MODIFIKASI 2: Menambahkan implementasi `From` untuk error baru ---

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::SqlxError(err)
    }
}

// Implementasi untuk error dari bcrypt
impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::BcryptError(err)
    }
}

// Implementasi untuk error dari jsonwebtoken
impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::JsonWebTokenError(err)
    }
}

// Implementasi untuk error dari anyhow
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

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            // --- BLOK INI TIDAK SAYA UBAH SAMA SEKALI, KARENA INI SUDAH BEKERJA UNTUK ANDA ---
            AppError::SqlxError(err) => {
                // Ubah error sqlx menjadi string untuk dianalisis
                let err_string = err.to_string();

                // Cek apakah ini adalah error unique constraint
                if err_string.contains("violates unique constraint") {
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
                    } else {
                        // Pesan fallback jika constraint tidak dikenali
                        "Data yang Anda masukkan sudah ada di sistem (nilai duplikat).".to_string()
                    };

                    // Kembalikan 409 Conflict dengan pesan yang sudah diterjemahkan
                    (StatusCode::CONFLICT, message)
                } else {
                    // Untuk semua error database lainnya
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Terjadi masalah pada database.".to_string(),
                    )
                }
            }
            // --- AKHIR DARI BLOK YANG TIDAK DIUBAH ---
            AppError::DuplicateEntry(message) => (StatusCode::CONFLICT, message.clone()),
            // --- MODIFIKASI 3: Menambahkan cabang `match` untuk error baru ---
            AppError::BcryptError(err) => {
                eprintln!("--> Bcrypt Error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Terjadi kesalahan pemrosesan internal.".to_string(),
                )
            }
            AppError::JsonWebTokenError(err) => {
                eprintln!("--> JWT Error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Terjadi kesalahan pada token otentikasi.".to_string(),
                )
            }
            AppError::AnyhowError(err) => {
                eprintln!("--> Logic Error: {:?}", err);
                // Untuk "Username atau password salah", kita gunakan 401 Unauthorized
                (StatusCode::UNAUTHORIZED, err.to_string())
            }
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "Anda tidak memiliki hak akses untuk sumber daya ini.".to_string(),
            ),
            AppError::MultipartError(err) => {
                eprintln!("--> Multipart Error: {:?}", err);
                (
                    StatusCode::BAD_REQUEST, // 400 Bad Request cocok untuk upload yang salah/gagal
                    format!("Request upload tidak valid: {}", err),
                )
            }
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
