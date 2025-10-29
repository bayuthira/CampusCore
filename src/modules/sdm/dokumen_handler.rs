// src/modules/sdm/dokumen_handler.rs
use super::dokumen_model::{DokumenSdmDetail, KategoriDokumen, SdmEntityType, DokumenFilter,DokumenSdmDetailAll};
use super::dokumen_repo;
use crate::{modules::auth::middleware::TokenClaims, db::DbPool, errors::AppError, utils::file_validator::validate_file};
use axum::{
    extract::{Multipart, Path, State, Query},
    http::StatusCode,
    Extension, Json,
};
use std::path::Path as StdPath;
use uuid::Uuid;
use std::ffi::OsStr;

pub async fn upload_dokumen_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path((entity_type_str, entity_id)): Path<(String, Uuid)>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<DokumenSdmDetail>), AppError> {
    
    let user_uploader_id = claims.sub;

    // 1. Validasi Tipe Entitas dari URL
    let entity_type = SdmEntityType::from_str(&entity_type_str)
        .ok_or_else(|| AppError::Forbidden(format!("Tipe entitas '{}' tidak valid.", entity_type_str)))?;

    // 2. TENTUKAN PEGAWAI ID (Perbaikan Logika)
    let pegawai_id: Uuid = match entity_type {
        SdmEntityType::Pegawai => {
            // Jika entitasnya Pegawai, entity_id adalah pegawai_id
            entity_id
        }
        SdmEntityType::RiwayatPendidikan => {
            // Jika entitasnya RiwayatPendidikan, cari pegawai_id-nya
            sqlx::query_scalar!(
                "SELECT pegawai_id FROM riwayat_pendidikan WHERE id = $1",
                entity_id
            )
            .fetch_optional(&pool)
            .await?
            .ok_or_else(|| AppError::Forbidden("Riwayat Pendidikan tidak ditemukan.".to_string()))?
        }
        SdmEntityType::RiwayatSk => {
            // Jika entitasnya RiwayatSk, cari pegawai_id-nya
            sqlx::query_scalar!(
                "SELECT pegawai_id FROM riwayat_sk WHERE id = $1",
                entity_id
            )
            .fetch_optional(&pool)
            .await?
            .ok_or_else(|| AppError::Forbidden("Riwayat SK tidak ditemukan.".to_string()))?
        }
    };

    // 3. Ambil data dari form
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut kategori_str: Option<String> = None;

    while let Some(field) = multipart.next_field().await? {
        let field_name = match field.name() {
            Some(name) => name.to_string(),
            None => continue,
        };

        match field_name.as_str() {
            "file" => {
                file_name = Some(field.file_name().unwrap_or("unknown_file").to_string());
                file_data = Some(field.bytes().await?.to_vec());
            }
            "kategori" => {
                kategori_str = Some(field.text().await?);
            }
            _ => {}
        }
    }

    // 4. Validasi input
    let data = file_data.ok_or_else(|| AppError::Forbidden("Field 'file' wajib ada.".to_string()))?;
    let nama_file_asli = file_name.ok_or_else(|| AppError::Forbidden("Nama file tidak ditemukan.".to_string()))?;
    let kategori_str = kategori_str.ok_or_else(|| AppError::Forbidden("Field 'kategori' wajib ada.".to_string()))?;
    
    let kategori = KategoriDokumen::from_str(&kategori_str)
        .ok_or_else(|| AppError::Forbidden(format!("Kategori '{}' tidak valid.", kategori_str)))?;
    
    // 5. Validasi Tipe File (MIME)
    validate_file(&data, &["image/jpeg", "image/png", "application/pdf"])?;
    let tipe_mime = infer::get(&data).map_or("application/octet-stream".to_string(), |k| k.mime_type().to_string());

    // 6. Buat path penyimpanan (Gunakan pegawai_id untuk subfolder)
    let file_extension = StdPath::new(&nama_file_asli).extension().and_then(OsStr::to_str).unwrap_or("");
    let new_file_name = format!("{}.{}", Uuid::new_v4(), file_extension);
    let file_path_str = format!("uploads/sdm/{}/{}", pegawai_id, new_file_name);

    // 7. Simpan file
    if let Some(parent) = StdPath::new(&file_path_str).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&file_path_str, data).await?;

    // 8. Simpan metadata ke database (sekarang dengan pegawai_id yang benar)
    let new_dokumen = dokumen_repo::create_dokumen_repo(
        &pool, pegawai_id, entity_id, entity_type, kategori, 
        nama_file_asli, file_path_str, tipe_mime, user_uploader_id
    ).await?;
    
    Ok((StatusCode::CREATED, Json(new_dokumen)))
}

pub async fn get_all_dokumen_handler(
    State(pool): State<DbPool>,
    Path((entity_type_str, entity_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<DokumenSdmDetail>>, AppError> {
    let entity_type = SdmEntityType::from_str(&entity_type_str)
        .ok_or_else(|| AppError::Forbidden(format!("Tipe entitas '{}' tidak valid.", entity_type_str)))?;
    
    let list = dokumen_repo::get_all_dokumen_by_entity_repo(&pool, entity_id, entity_type).await?;
    Ok(Json(list))
}

pub async fn delete_dokumen_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>, // ID dari dokumen
) -> Result<StatusCode, AppError> {
    dokumen_repo::delete_dokumen_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}


pub async fn get_all_dokumen_admin_handler(
    State(pool): State<DbPool>,
    Query(filter): Query<DokumenFilter>,
) -> Result<Json<Vec<DokumenSdmDetailAll>>, AppError> {
    let list = dokumen_repo::get_all_dokumen_repo(&pool, filter).await?;
    Ok(Json(list))
}