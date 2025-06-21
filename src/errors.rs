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
            AppError::SqlxError(sqlx::Error::Database(db_err)) => {
                if let Some(pg_err) = db_err
                    .as_ref()
                    .try_downcast_ref::<sqlx::postgres::PgDatabaseError>()
                {
                    match pg_err.code() {
                        "23505" => (
                            StatusCode::CONFLICT,
                            "Data dengan kunci tersebut sudah ada.".to_string(),
                        ),
                        _ => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Terjadi masalah pada database: {}", pg_err),
                        ),
                    }
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Terjadi masalah tak terduga pada database.".to_string(),
                    )
                }
            }
            AppError::SqlxError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Terjadi masalah internal sistem.".to_string(),
            ),
            // --- AKHIR DARI BLOK YANG TIDAK DIUBAH ---

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
