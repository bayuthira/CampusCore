// src/modules/sdm/absensi_wajah_handler.rs
use super::{absensi_wajah_repo, repo as pegawai_repo};
use crate::{
    db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims,
    utils::file_validator::validate_file,
};
use axum::{
    Extension, Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::Path as StdPath;
use uuid::Uuid;

#[derive(Serialize)]
pub struct EnrollResponse {
    pub message: String,
    pub status_audit: String,
}

#[derive(Deserialize)]
pub struct AuditWajahPayload {
    pub status_audit: String, // "Valid" atau "Ditolak"
}

// =========================================================================
// 1. UPLOAD FOTO WAJAH (SELF-ENROLLMENT)
// =========================================================================
pub async fn enroll_wajah_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<EnrollResponse>), AppError> {
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, claims.sub).await?;

    let (existing_foto, _) = absensi_wajah_repo::get_status_wajah_repo(&pool, pegawai_id).await?;
    if existing_foto.is_some() {
        return Err(AppError::Forbidden(
            "Foto referensi wajah Anda sudah terdaftar di sistem. Jika ingin mengubahnya, silakan hubungi Admin SDM.".to_string()
        ));
    }

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            file_name = Some(field.file_name().unwrap_or("selfie.jpg").to_string());
            file_data = Some(field.bytes().await?.to_vec());
        }
    }

    let data = file_data
        .ok_or_else(|| AppError::BadRequest("Field 'file' foto wajib diunggah.".to_string()))?;
    let nama_file_asli = file_name.unwrap();

    validate_file(&data, &["image/jpeg", "image/png"])?;

    let file_extension = StdPath::new(&nama_file_asli)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("jpg");

    let new_file_name = format!("ref_{}.{}", Uuid::new_v4(), file_extension);
    let file_path_str = format!("uploads/sdm/wajah/{}/{}", pegawai_id, new_file_name);

    if let Some(parent) = StdPath::new(&file_path_str).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&file_path_str, &data).await?;

    absensi_wajah_repo::enroll_wajah_repo(&pool, pegawai_id, file_path_str).await?;

    Ok((
        StatusCode::OK,
        Json(EnrollResponse {
            message: "Foto referensi wajah berhasil diunggah dan siap digunakan.".to_string(),
            status_audit: "Menunggu Audit".to_string(),
        }),
    ))
}

// =========================================================================
// 2.A VIEW FOTO WAJAH SENDIRI (Khusus Karyawan - Tanpa parameter ID URL)
// =========================================================================
pub async fn get_my_foto_wajah_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
) -> Result<impl IntoResponse, AppError> {
    // Otomatis cari Pegawai ID dari token Karyawan yang login
    let pegawai_id = match pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, claims.sub).await {
        Ok(id) => id,
        Err(_) => {
            return Err(AppError::Forbidden(
                "Akun Anda belum direlasikan dengan data Pegawai di sistem.".to_string(),
            ));
        }
    };

    let (path_opt, _) = absensi_wajah_repo::get_status_wajah_repo(&pool, pegawai_id).await?;
    let file_path = path_opt
        .ok_or_else(|| AppError::BadRequest("Foto referensi wajah belum tersedia.".to_string()))?;

    let data = tokio::fs::read(&file_path).await.map_err(|_| {
        AppError::InternalServerError("Gagal membaca file foto fisik dari server.".to_string())
    })?;

    let mime_type = infer::get(&data).map_or("image/jpeg", |k| k.mime_type());

    Ok(([(axum::http::header::CONTENT_TYPE, mime_type)], data))
}

// =========================================================================
// 2.B VIEW FOTO WAJAH SPESIFIK (Untuk Admin yg melihat list karyawan)
// =========================================================================
pub async fn get_foto_wajah_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(pegawai_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let is_admin = claims.roles.contains(&"SUPER_ADMIN".to_string())
        || claims.roles.contains(&"STAF_BASDM".to_string());
    if !is_admin {
        let logged_in_pegawai_id =
            match pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, claims.sub).await {
                Ok(id) => id,
                Err(_) => {
                    return Err(AppError::Forbidden(
                        "Akun Anda belum direlasikan dengan data Pegawai di sistem.".to_string(),
                    ));
                }
            };

        if logged_in_pegawai_id != pegawai_id {
            return Err(AppError::Forbidden(
                "Anda hanya dapat melihat foto wajah milik Anda sendiri.".to_string(),
            ));
        }
    }

    let (path_opt, _) = absensi_wajah_repo::get_status_wajah_repo(&pool, pegawai_id).await?;
    let file_path = path_opt
        .ok_or_else(|| AppError::BadRequest("Foto referensi wajah belum tersedia.".to_string()))?;

    let data = tokio::fs::read(&file_path).await.map_err(|_| {
        AppError::InternalServerError("Gagal membaca file foto fisik dari server (Mungkin file terhapus atau berbeda server).".to_string())
    })?;

    let mime_type = infer::get(&data).map_or("image/jpeg", |k| k.mime_type());

    Ok(([(axum::http::header::CONTENT_TYPE, mime_type)], data))
}

// =========================================================================
// 3. AUDIT / VERIFIKASI FOTO WAJAH (Khusus Admin)
// =========================================================================
pub async fn audit_wajah_handler(
    State(pool): State<DbPool>,
    Path(pegawai_id): Path<Uuid>,
    Json(payload): Json<AuditWajahPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    if payload.status_audit != "Valid" && payload.status_audit != "Ditolak" {
        return Err(AppError::BadRequest(
            "Status audit harus 'Valid' atau 'Ditolak'.".to_string(),
        ));
    }

    absensi_wajah_repo::audit_wajah_repo(&pool, pegawai_id, payload.status_audit.clone()).await?;

    Ok(Json(serde_json::json!({
        "message": format!("Status foto referensi wajah berhasil diperbarui menjadi: {}", payload.status_audit)
    })))
}

// =========================================================================
// 4. HAPUS FOTO WAJAH (Khusus Admin)
// =========================================================================
pub async fn delete_wajah_handler(
    State(pool): State<DbPool>,
    Path(pegawai_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let (path_opt, _) = absensi_wajah_repo::get_status_wajah_repo(&pool, pegawai_id).await?;

    if let Some(file_path) = path_opt {
        let _ = tokio::fs::remove_file(&file_path).await;
    }

    absensi_wajah_repo::delete_wajah_repo(&pool, pegawai_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
