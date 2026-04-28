// src/modules/akademik/rencana_penilaian_handler.rs
use super::{
    rencana_penilaian_model::{RencanaPenilaianDetail, UpsertRencanaPenilaianPayload},
    rencana_penilaian_repo as repo,
};
use crate::{db::DbPool, errors::AppError};
use axum::{
    extract::{Json, Multipart, Path, State},
    http::StatusCode,
};
use std::ffi::OsStr;
use std::path::Path as StdPath;
use uuid::Uuid;

/// Handler untuk melihat rencana penilaian di suatu jadwal/kelas
pub async fn get_rencana_penilaian_handler(
    State(pool): State<DbPool>,
    Path(jadwal_kuliah_id): Path<Uuid>,
) -> Result<Json<Option<RencanaPenilaianDetail>>, AppError> {
    let rencana = repo::get_rencana_penilaian_by_jadwal_repo(&pool, jadwal_kuliah_id).await?;
    Ok(Json(rencana))
}

/// Handler untuk menyimpan/update persentase bobot penilaian (Wajib 100%)
pub async fn upsert_rencana_penilaian_handler(
    State(pool): State<DbPool>,
    Path(jadwal_kuliah_id): Path<Uuid>,
    Json(payload): Json<UpsertRencanaPenilaianPayload>,
) -> Result<(StatusCode, Json<RencanaPenilaianDetail>), AppError> {
    let rencana = repo::upsert_rencana_penilaian_repo(&pool, jadwal_kuliah_id, payload).await?;
    Ok((StatusCode::OK, Json(rencana)))
}

/// Handler untuk mengupload file dokumen (Kontrak Kuliah / Praktikum)
pub async fn upload_file_rencana_penilaian_handler(
    State(pool): State<DbPool>,
    Path((jadwal_kuliah_id, jenis_file)): Path<(Uuid, String)>,
    mut multipart: Multipart,
) -> Result<StatusCode, AppError> {
    // Validasi parameter URL
    let kolom_db = match jenis_file.as_str() {
        "kontrak" => "file_kontrak_path",
        "praktikum" => "file_praktikum_path",
        _ => {
            return Err(AppError::BadRequest(
                "Jenis file harus 'kontrak' atau 'praktikum'.".to_string(),
            ));
        }
    };

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
    let file_name = original_name.unwrap_or_else(|| "upload.pdf".to_string());

    // Generate path penyimpanan yang rapi
    let ext = StdPath::new(&file_name)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("pdf");
    let new_filename = format!("{}.{}", Uuid::new_v4(), ext);
    let folder_path = format!("uploads/akademik/rencana_penilaian/{}", jadwal_kuliah_id);
    let save_path = format!("{}/{}", folder_path, new_filename);

    tokio::fs::create_dir_all(&folder_path).await?;
    tokio::fs::write(&save_path, data).await?;

    // Update path ke Database
    repo::update_file_rencana_penilaian_repo(&pool, jadwal_kuliah_id, kolom_db, save_path).await?;

    Ok(StatusCode::OK)
}
