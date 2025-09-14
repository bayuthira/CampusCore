// src/modules/aset/biaya_handler.rs
use crate::{
    modules::auth::middleware::TokenClaims,
    db::DbPool,
    errors::AppError,
    modules::aset::biaya_model::{BiayaAset, BiayaAsetPayload,TipeBiaya},
    modules::aset::biaya_repo,
};
use axum::{
    extract::{Path, State, Json,Multipart},
    http::StatusCode,
    Extension,
};
use uuid::Uuid;
use rust_decimal::Decimal;
use time::Date;

pub async fn create_biaya_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<BiayaAset>), AppError> {
    let user_pencatat_id = claims.sub;
    
    let mut aset_id: Option<Uuid> = None;
    let mut tipe_biaya: Option<TipeBiaya> = None;
    let mut deskripsi: Option<String> = None;
    let mut jumlah: Option<Decimal> = None;
    let mut tanggal_transaksi: Option<Date> = None;
    let mut vendor: Option<String> = None;
    let mut bukti_url: Option<String> = None;

    while let Some(field) = multipart.next_field().await? {
        // --- PERBAIKAN DI SINI ---
        // Ambil nama field dan langsung kloning menjadi String.
        // Ini akan menyelesaikan pinjaman (borrow) pada `field`.
        let field_name = match field.name() {
            Some(name) => name.to_string(),
            None => continue, // Lewati field tanpa nama
        };

        match field_name.as_str() {
            "bukti" => {
                let file_name = field.file_name().unwrap_or("unknown_file").to_string();
                let file_extension = std::path::Path::new(&file_name).extension().and_then(std::ffi::OsStr::to_str).unwrap_or("");
                let new_file_name = format!("{}.{}", Uuid::new_v4(), file_extension);
                let file_path_str = format!("uploads/bukti_biaya/{}", new_file_name);
                
                tokio::fs::create_dir_all("uploads/bukti_biaya").await?;
                // Sekarang `.bytes()` bisa mengambil kepemilikan `field` tanpa masalah.
                let data = field.bytes().await?;
                tokio::fs::write(&file_path_str, data).await?;
                
                bukti_url = Some(file_path_str);
            }
            // Untuk field teks, `.text()` juga mengambil kepemilikan `field`.
            _ => {
                let value = field.text().await?;
                match field_name.as_str() {
                    "aset_id" => aset_id = Some(Uuid::parse_str(&value)?),
                    "tipe_biaya" => tipe_biaya = Some(serde_json::from_str(&format!("\"{}\"", value))?),
                    "deskripsi" => deskripsi = Some(value),
                    "jumlah" => jumlah = Some(value.parse()?),
                    "tanggal_transaksi" => tanggal_transaksi = Some(time::Date::parse(&value, &time::format_description::well_known::Iso8601::DEFAULT)?),
                    "vendor" => vendor = Some(value),
                    _ => {}
                }
            }
        }
    }

    // Validasi bahwa semua field wajib sudah diisi
    let payload = BiayaAsetPayload {
        aset_id: aset_id.ok_or_else(|| AppError::AnyhowError(anyhow::anyhow!("Field 'aset_id' wajib diisi.")))?,
        tipe_biaya: tipe_biaya.ok_or_else(|| AppError::AnyhowError(anyhow::anyhow!("Field 'tipe_biaya' wajib diisi.")))?,
        deskripsi: deskripsi.ok_or_else(|| AppError::AnyhowError(anyhow::anyhow!("Field 'deskripsi' wajib diisi.")))?,
        jumlah: jumlah.ok_or_else(|| AppError::AnyhowError(anyhow::anyhow!("Field 'jumlah' wajib diisi.")))?,
        tanggal_transaksi: tanggal_transaksi.ok_or_else(|| AppError::AnyhowError(anyhow::anyhow!("Field 'tanggal_transaksi' wajib diisi.")))?,
        vendor,
    };

    let new_biaya = biaya_repo::create_biaya_repo(&pool, user_pencatat_id, payload, bukti_url).await?;
    Ok((StatusCode::CREATED, Json(new_biaya)))
}


pub async fn get_all_biaya_by_aset_id_handler(
    State(pool): State<DbPool>,
    Path(aset_id): Path<Uuid>,
) -> Result<Json<Vec<BiayaAset>>, AppError> {
    let list = biaya_repo::get_all_biaya_by_aset_id_repo(&pool, aset_id).await?;
    Ok(Json(list))
}

pub async fn update_biaya_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>, // ID dari biaya_aset
    Json(payload): Json<BiayaAsetPayload>,
) -> Result<Json<BiayaAset>, AppError> {
    let user_pencatat_id = claims.sub;
    let updated_biaya = biaya_repo::update_biaya_repo(&pool, id, user_pencatat_id, payload).await?;
    Ok(Json(updated_biaya))
}

pub async fn delete_biaya_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>, // ID dari biaya_aset
) -> Result<StatusCode, AppError> {
    biaya_repo::delete_biaya_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}