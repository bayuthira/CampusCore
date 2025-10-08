use super::{
    model::{AsetDetail, AsetPayload,HistoriAsetDetail,PindahkanAsetPayload,UpdateKondisiPayload,CreateHistoriPayload,PinjamAsetPayload, KembalikanAsetPayload,AsetFilter,KondisiAsetSummary,AktivitasSummary},
    repo as aset_repo,
    histori_repo as histori_aset_repo,
};
use crate::modules::general::model::SuccessResponse;

use crate::{
    db::DbPool,
    errors::AppError,
    modules::auth::middleware::TokenClaims,
};
use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;
use axum::Extension;

/// Handler untuk membuat Aset baru
pub async fn create_aset_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<AsetPayload>,
) -> Result<(StatusCode, Json<AsetDetail>), AppError> {
    let aset = aset_repo::create_aset_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(aset)))
}

/// Handler untuk mendapatkan semua Aset

pub async fn get_all_aset_handler(
    State(pool): State<DbPool>,
    Query(filter): Query<AsetFilter>, // <-- Terima filter dari query parameter
) -> Result<Json<Vec<AsetDetail>>, AppError> {
    // Teruskan filter ke repository
    let list = aset_repo::get_all_aset_repo(&pool, filter).await?;
    Ok(Json(list))
}

/// Handler untuk mendapatkan satu Aset berdasarkan ID
pub async fn get_aset_by_id_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<AsetDetail>, AppError> {
    let aset = aset_repo::get_aset_by_id_repo(&pool, id).await?;
    Ok(Json(aset))
}

/// Handler untuk memperbarui Aset
pub async fn update_aset_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AsetPayload>,
) -> Result<Json<AsetDetail>, AppError> {
    let updated = aset_repo::update_aset_repo(&pool, id, payload).await?;
    Ok(Json(updated))
}

/// Handler untuk menghapus Aset
pub async fn delete_aset_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    aset_repo::delete_aset_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_aset_histori_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>, // ID dari aset
) -> Result<Json<Vec<HistoriAsetDetail>>, AppError> {
    let histori = histori_aset_repo::get_histori_by_aset_id_repo(&pool, id).await?;
    Ok(Json(histori))
}

pub async fn pindahkan_aset_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>, // Ambil info user yang login
    Path(aset_id): Path<Uuid>,
    Json(payload): Json<PindahkanAsetPayload>,
) -> Result<Json<AsetDetail>, AppError> {
    let user_aksi_id = claims.sub; // ID user yang melakukan aksi
    let aset_terbaru =
        histori_aset_repo::pindahkan_aset_repo(&pool, aset_id, user_aksi_id, payload).await?;
    Ok(Json(aset_terbaru))
}

pub async fn update_kondisi_aset_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(aset_id): Path<Uuid>,
    Json(payload): Json<UpdateKondisiPayload>,
) -> Result<Json<AsetDetail>, AppError> {
    let user_aksi_id = claims.sub;
    let aset_terbaru =
        histori_aset_repo::update_kondisi_aset_repo(&pool, aset_id, user_aksi_id, payload).await?;
    Ok(Json(aset_terbaru))
}


pub async fn create_histori_aset_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(aset_id): Path<Uuid>,
    Json(payload): Json<CreateHistoriPayload>,
) -> Result<Json<AsetDetail>, AppError> {
    let user_aksi_id = claims.sub;
    let aset_terbaru =
        histori_aset_repo::create_histori_repo(&pool, aset_id, user_aksi_id, payload).await?;
    Ok(Json(aset_terbaru))
}


pub async fn pinjam_aset_handler(State(pool): State<DbPool>, Extension(claims): Extension<TokenClaims>, Path(aset_id): Path<Uuid>, Json(payload): Json<PinjamAsetPayload>) -> Result<Json<SuccessResponse>, AppError> {
    let user_approve_id = claims.sub;
    histori_aset_repo::pinjam_aset_repo(&pool, aset_id, user_approve_id, payload).await?;
    Ok(Json(SuccessResponse { message: "Aset berhasil dicatat sebagai dipinjam.".to_string() }))
}

pub async fn kembalikan_aset_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(peminjaman_id): Path<Uuid>, // <-- Diubah dari aset_id ke peminjaman_id
    Json(payload): Json<KembalikanAsetPayload>,
) -> Result<Json<SuccessResponse>, AppError> {
    let user_approve_id = claims.sub;
    histori_aset_repo::kembalikan_aset_repo(&pool, peminjaman_id, user_approve_id, payload).await?;
    Ok(Json(SuccessResponse {
        message: "Aset berhasil dicatat sebagai dikembalikan.".to_string(),
    }))
}

pub async fn get_kondisi_summary_handler(State(pool): State<DbPool>) -> Result<Json<KondisiAsetSummary>, AppError> {
    let summary = aset_repo::get_kondisi_summary_repo(&pool).await?;
    Ok(Json(summary))
}

pub async fn get_aktivitas_summary_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match histori_aset_repo::get_aktivitas_summary_repo(&pool, id).await {
        Ok(summary) => Json(summary).into_response(),
        Err(e) => e.into_response(),
    }
}