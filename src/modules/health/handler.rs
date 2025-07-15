// src/handlers/health_handler.rs
use axum::{extract::State, http::StatusCode};
use crate::db::DbPool;
use crate::errors::AppError;

/// Handler untuk health check.
/// Mengekstrak `DbPool` dari state aplikasi.
pub async fn health_check(
    State(pool): State<DbPool>,
) -> Result<StatusCode, AppError> {
    // Menjalankan query sederhana untuk memeriksa koneksi database.
    // `sqlx::query!` akan divalidasi saat kompilasi jika `sqlx-cli` disetup.
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await?; // Operator `?` akan otomatis mengkonversi sqlx::Error menjadi AppError

    // Jika query berhasil, kembalikan status OK
    Ok(StatusCode::OK)
}