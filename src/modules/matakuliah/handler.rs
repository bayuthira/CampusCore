// src/modules/matakuliah/handler.rs

use super::{
    model::{
        CreateMataKuliahPayload, MataKuliahDetail, UpdateMataKuliahPayload, VerifikasiRpsPayload,
    },
    repo as matakuliah_repo,
};

use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};

use axum::{
    Extension,
    body::Body,
    extract::{Json, Multipart, Path, State},
    http::{Response, StatusCode, header},
};
use std::ffi::OsStr;
use std::path::Path as StdPath; // <-- TAMBAHKAN INI
use uuid::Uuid; // <-- TAMBAHKAN INI

/// Handler untuk membuat Mata Kuliah baru
pub async fn create_matakuliah_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateMataKuliahPayload>,
) -> Result<(StatusCode, Json<MataKuliahDetail>), AppError> {
    let new_mk = matakuliah_repo::create_matakuliah_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(new_mk)))
}

/// Handler untuk mendapatkan semua Mata Kuliah
pub async fn get_all_matakuliah_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<MataKuliahDetail>>, AppError> {
    let mk_list = matakuliah_repo::get_all_matakuliah_repo(&pool).await?;
    Ok(Json(mk_list))
}

/// Handler untuk mendapatkan satu Mata Kuliah berdasarkan ID
pub async fn get_matakuliah_by_id_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<MataKuliahDetail>, AppError> {
    let mk = matakuliah_repo::get_matakuliah_by_id_repo(&pool, id).await?;
    Ok(Json(mk))
}

/// Handler untuk memperbarui Mata Kuliah
pub async fn update_matakuliah_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<TokenClaims>, // <-- Ambil info user
    Json(payload): Json<UpdateMataKuliahPayload>,
) -> Result<Json<MataKuliahDetail>, AppError> {
    // Cek apakah ada upaya untuk mengubah Kode MK
    if let Some(ref _kode_mk) = payload.kode_mk {
        // Jika ada, hanya SUPER_ADMIN yang boleh melanjutkan
        if !claims.roles.contains(&"SUPER_ADMIN".to_string()) {
            return Err(AppError::Forbidden(
                "Hanya SUPER_ADMIN yang dapat mengubah Kode MK.".to_string(),
            ));
        }
    }

    let updated_mk = matakuliah_repo::update_matakuliah_repo(&pool, id, payload).await?;
    Ok(Json(updated_mk))
}

/// Handler untuk menghapus Mata Kuliah
pub async fn delete_matakuliah_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    matakuliah_repo::delete_matakuliah_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn verifikasi_rps_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<VerifikasiRpsPayload>,
) -> Result<Json<MataKuliahDetail>, AppError> {
    let updated_mk = matakuliah_repo::verifikasi_rps_repo(&pool, id, payload).await?;
    Ok(Json(updated_mk))
}

pub async fn upload_file_rps_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<
    (
        StatusCode,
        Json<crate::modules::general::model::SuccessResponse>,
    ),
    AppError,
> {
    let mut file_data = None;
    let mut original_name = None;

    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            original_name = field.file_name().map(|s| s.to_string());
            file_data = Some(field.bytes().await?.to_vec());
        }
    }

    let data =
        file_data.ok_or_else(|| AppError::BadRequest("Field 'file' wajib diisi.".to_string()))?;
    let file_name = original_name.unwrap_or_else(|| "rps.pdf".to_string());

    const MAX_FILE_SIZE: usize = 5 * 1024 * 1024;
    if data.len() > MAX_FILE_SIZE {
        return Err(AppError::BadRequest(
            "Ukuran file RPS maksimal 5 MB.".to_string(),
        ));
    }

    // Ekstrak ekstensi file (.pdf, .doc, .docx)
    let ext = StdPath::new(&file_name)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_ascii_lowercase();

    let detected_mime = infer::get(&data).map(|kind| kind.mime_type()).unwrap_or("");
    let valid_type = match ext.as_str() {
        "pdf" => detected_mime == "application/pdf",
        "doc" => {
            detected_mime == "application/x-ole-storage" || detected_mime == "application/msword"
        }
        "docx" => {
            detected_mime == "application/zip"
                || detected_mime
                    == "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        }
        _ => false,
    };

    if !valid_type {
        return Err(AppError::BadRequest(
            "File RPS harus berupa PDF, DOC, atau DOCX yang valid.".to_string(),
        ));
    }

    // Simpan file secara rapi per UUID Mata Kuliah
    let new_filename = format!("{}.{}", Uuid::new_v4(), ext);
    let folder_path = format!("uploads/akademik/rps/{}", id);
    let save_path = format!("{}/{}", folder_path, new_filename);

    tokio::fs::create_dir_all(&folder_path).await?;
    tokio::fs::write(&save_path, data).await?;

    // Update kolom `file_rps_path` di tabel mata_kuliah (Otomatis merubah status ke Menunggu Verifikasi)
    let old_path = match matakuliah_repo::update_file_rps_repo(&pool, id, save_path.clone()).await {
        Ok(path) => path,
        Err(error) => {
            let _ = tokio::fs::remove_file(&save_path).await;
            return Err(error);
        }
    };

    if let Some(old_path) = old_path {
        if old_path != save_path {
            let _ = tokio::fs::remove_file(old_path).await;
        }
    }

    Ok((
        StatusCode::OK,
        Json(crate::modules::general::model::SuccessResponse {
            message: "File RPS berhasil diunggah. Menunggu verifikasi Kaprodi.".to_string(),
        }),
    ))
}

pub async fn get_file_rps_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Response<Body>, AppError> {
    let mata_kuliah = matakuliah_repo::get_matakuliah_by_id_repo(&pool, id).await?;
    let file_path = mata_kuliah
        .file_rps_path
        .ok_or_else(|| AppError::BadRequest("Dokumen RPS belum diunggah.".to_string()))?;

    let canonical_uploads = tokio::fs::canonicalize("./uploads").await?;
    let canonical_file = tokio::fs::canonicalize(&file_path).await?;
    if !canonical_file.starts_with(canonical_uploads) || !canonical_file.is_file() {
        return Err(AppError::Forbidden(
            "Path dokumen RPS tidak valid.".to_string(),
        ));
    }

    let file_contents = tokio::fs::read(&canonical_file).await?;
    let mime_type = mime_guess::from_path(&canonical_file)
        .first_or_octet_stream()
        .to_string();
    let extension = canonical_file
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("bin");
    let safe_code: String = mata_kuliah
        .kode_mk
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || *character == '-' || *character == '_'
        })
        .collect();
    let download_name = format!("RPS-{}.{}", safe_code, extension);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", download_name),
        )
        .body(Body::from(file_contents))
        .map_err(|error| AppError::InternalServerError(error.to_string()))?)
}
