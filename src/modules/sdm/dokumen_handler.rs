// src/modules/sdm/dokumen_handler.rs
use super::dokumen_model::{DokumenSdmDetail, KategoriDokumen, SdmEntityType, DokumenFilter,DokumenSdmDetailAll};
use super::dokumen_repo;
use super::repo as pegawai_repo; // <-- Tambahan import untuk mengambil ID Pegawai
use crate::{modules::auth::middleware::TokenClaims, db::DbPool, errors::AppError, utils::file_validator::validate_file};
use axum::{
    extract::{Multipart, Path, State, Query},
    http::StatusCode,
    Extension, Json,
};
use std::path::Path as StdPath;
use uuid::Uuid;
use std::ffi::OsStr;

// --- HELPER UNTUK MENCARI PEGAWAI ID DARI ENTITAS ---
async fn resolve_pegawai_id(pool: &DbPool, entity_type: &SdmEntityType, entity_id: Uuid) -> Result<Uuid, AppError> {
    match entity_type {
        SdmEntityType::Pegawai => Ok(entity_id),
        SdmEntityType::RiwayatPendidikan => {
            sqlx::query_scalar!("SELECT pegawai_id FROM riwayat_pendidikan WHERE id = $1", entity_id)
                .fetch_optional(pool).await?
                .ok_or_else(|| AppError::Forbidden("Riwayat Pendidikan tidak ditemukan.".to_string()))
        }
        SdmEntityType::RiwayatSk => {
            sqlx::query_scalar!("SELECT pegawai_id FROM riwayat_sk WHERE id = $1", entity_id)            
                .fetch_optional(pool).await?
                .ok_or_else(|| AppError::Forbidden("Riwayat SK tidak ditemukan.".to_string()))
        }
        SdmEntityType::RiwayatSertifikat => {
            sqlx::query_scalar!("SELECT pegawai_id FROM riwayat_sertifikat WHERE id = $1", entity_id)
                .fetch_optional(pool).await?
                .ok_or_else(|| AppError::Forbidden("Riwayat Sertifikat tidak ditemukan.".to_string()))
        }
        SdmEntityType::RiwayatJad => {
            sqlx::query_scalar!("SELECT pegawai_id FROM riwayat_jad WHERE id = $1", entity_id)
                .fetch_optional(pool).await?
                .ok_or_else(|| AppError::Forbidden("Riwayat JAD tidak ditemukan.".to_string()))
        }
        SdmEntityType::RiwayatSerdos => {
            sqlx::query_scalar!("SELECT pegawai_id FROM riwayat_serdos WHERE id = $1", entity_id)
                .fetch_optional(pool).await?
                .ok_or_else(|| AppError::Forbidden("Riwayat SERDOS tidak ditemukan.".to_string()))
        }
        SdmEntityType::PengajuanIjin => {
            sqlx::query_scalar!("SELECT pegawai_id FROM pengajuan_ijin WHERE id = $1", entity_id)
                .fetch_optional(pool).await?
                .ok_or_else(|| AppError::Forbidden("Pengajuan Ijin tidak ditemukan.".to_string()))
        }
    }
}

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

    // 2. Tentukan Pegawai ID menggunakan Helper
    let target_pegawai_id = resolve_pegawai_id(&pool, &entity_type, entity_id).await?;

    // 3. LOGIKA OTORISASI: Karyawan hanya bisa upload untuk dirinya sendiri
    let is_admin = claims.roles.contains(&"SUPER_ADMIN".to_string()) || claims.roles.contains(&"STAF_BASDM".to_string());
    if !is_admin {
        let logged_in_pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_uploader_id).await?;
        if target_pegawai_id != logged_in_pegawai_id {
            return Err(AppError::Forbidden("Akses ditolak. Anda hanya dapat mengupload dokumen untuk data Anda sendiri.".to_string()));
        }
    }

    // 4. Ambil data dari form
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

    // 5. Validasi input
    let data = file_data.ok_or_else(|| AppError::Forbidden("Field 'file' wajib ada.".to_string()))?;
    let nama_file_asli = file_name.ok_or_else(|| AppError::Forbidden("Nama file tidak ditemukan.".to_string()))?;
    let kategori_str = kategori_str.ok_or_else(|| AppError::Forbidden("Field 'kategori' wajib ada.".to_string()))?;
    
    let kategori = KategoriDokumen::from_str(&kategori_str)
        .ok_or_else(|| AppError::Forbidden(format!("Kategori '{}' tidak valid.", kategori_str)))?;
    
    // 6. Validasi Tipe File (MIME)
    validate_file(&data, &["image/jpeg", "image/png", "application/pdf"])?;
    let tipe_mime = infer::get(&data).map_or("application/octet-stream".to_string(), |k| k.mime_type().to_string());

    // 7. Buat path penyimpanan (Gunakan target_pegawai_id untuk subfolder)
    let file_extension = StdPath::new(&nama_file_asli).extension().and_then(OsStr::to_str).unwrap_or("");
    let new_file_name = format!("{}.{}", Uuid::new_v4(), file_extension);
    let file_path_str = format!("uploads/sdm/{}/{}", target_pegawai_id, new_file_name);

    // 8. Simpan file
    if let Some(parent) = StdPath::new(&file_path_str).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&file_path_str, data).await?;

    // 9. Simpan metadata ke database
    let new_dokumen = dokumen_repo::create_dokumen_repo(
        &pool, target_pegawai_id, entity_id, entity_type, kategori, 
        nama_file_asli, file_path_str, tipe_mime, user_uploader_id
    ).await?;
    
    Ok((StatusCode::CREATED, Json(new_dokumen)))
}

pub async fn get_all_dokumen_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>, // <-- Tambahkan parameter klaim token
    Path((entity_type_str, entity_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<DokumenSdmDetail>>, AppError> {
    
    let entity_type = SdmEntityType::from_str(&entity_type_str)
        .ok_or_else(|| AppError::Forbidden(format!("Tipe entitas '{}' tidak valid.", entity_type_str)))?;
    
    // LOGIKA OTORISASI: Karyawan hanya boleh melihat dokumennya sendiri
    let is_admin = claims.roles.contains(&"SUPER_ADMIN".to_string()) || claims.roles.contains(&"STAF_BASDM".to_string());
    if !is_admin {
        let target_pegawai_id = resolve_pegawai_id(&pool, &entity_type, entity_id).await?;
        let logged_in_pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, claims.sub).await?;
        if target_pegawai_id != logged_in_pegawai_id {
            return Err(AppError::Forbidden("Akses ditolak. Anda hanya dapat melihat dokumen milik Anda sendiri.".to_string()));
        }
    }

    let list = dokumen_repo::get_all_dokumen_by_entity_repo(&pool, entity_id, entity_type).await?;
    Ok(Json(list))
}

pub async fn delete_dokumen_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>, // <-- Tambahkan parameter klaim token
    Path(id): Path<Uuid>, // ID dari dokumen
) -> Result<StatusCode, AppError> {
    
    // 1. Ambil dokumen dari DB untuk mengecek siapa pemiliknya
    let doc = dokumen_repo::get_dokumen_by_id_repo(&pool, id).await?;

    // 2. LOGIKA OTORISASI: Pastikan karyawan hanya bisa menghapus file miliknya
    let is_admin = claims.roles.contains(&"SUPER_ADMIN".to_string()) || claims.roles.contains(&"STAF_BASDM".to_string());
    if !is_admin {
        let logged_in_pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, claims.sub).await?;
        if doc.pegawai_id != logged_in_pegawai_id {
            return Err(AppError::Forbidden("Akses ditolak. Anda tidak berhak menghapus dokumen milik pegawai lain.".to_string()));
        }
    }

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