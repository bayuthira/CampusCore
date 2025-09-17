// src/utils/file_validator.rs
use crate::errors::AppError;

/// Memvalidasi tipe file berdasarkan kontennya.
///
/// # Arguments
/// * `file_bytes`: Konten file dalam bentuk bytes.
/// * `allowed_types`: Slice dari string yang berisi tipe MIME yang diizinkan,
///   contoh: `&["image/jpeg", "image/png", "application/pdf"]`.
///
/// # Returns
/// `Ok(true)` jika valid, `AppError` jika tidak valid.
pub fn validate_file(file_bytes: &[u8], allowed_types: &[&str]) -> Result<(), AppError> {
    // Deteksi tipe file dari beberapa byte pertama
    if let Some(kind) = infer::get(file_bytes) {
        let mime_type = kind.mime_type();
        // Cek apakah tipe yang terdeteksi ada di dalam daftar yang diizinkan
        if allowed_types.contains(&mime_type) {
            Ok(()) // Tipe file valid
        } else {
            let err_msg = format!(
                "Tipe file tidak valid. Terdeteksi: {}, Diizinkan: {}",
                mime_type,
                allowed_types.join(", ")
            );
            Err(AppError::Forbidden(err_msg))
        }
    } else {
        // Jika `infer` tidak bisa menebak, tolak untuk keamanan
        Err(AppError::Forbidden(
            "Tidak dapat menentukan tipe file.".to_string(),
        ))
    }
}