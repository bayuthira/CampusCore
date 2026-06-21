use super::{
    model::{
        CheckInMahasiswaPayload, CreatePertemuanPayload, DetailPertemuanResponse,
        KelasPembelajaran, ManualPresensiPayload, PertemuanKuliah, SesiPresensiResponse,
        SuccessMessage, UpdateBapPayload,
    },
    repo,
};
use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

async fn dosen_context(
    pool: &DbPool,
    claims: &TokenClaims,
    jadwal_id: Uuid,
) -> Result<Uuid, AppError> {
    let dosen_id = repo::get_dosen_id_by_user(pool, claims.sub).await?;
    repo::assert_dosen_access(pool, jadwal_id, dosen_id).await?;
    Ok(dosen_id)
}

async fn dosen_context_by_pertemuan(
    pool: &DbPool,
    claims: &TokenClaims,
    pertemuan_id: Uuid,
) -> Result<Uuid, AppError> {
    let jadwal_id = repo::get_jadwal_id_by_pertemuan(pool, pertemuan_id).await?;
    dosen_context(pool, claims, jadwal_id).await
}

pub async fn get_kelas_saya_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
) -> Result<Json<Vec<KelasPembelajaran>>, AppError> {
    let dosen_id = repo::get_dosen_id_by_user(&pool, claims.sub).await?;
    Ok(Json(repo::get_kelas_saya(&pool, dosen_id).await?))
}

pub async fn get_pertemuan_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(jadwal_id): Path<Uuid>,
) -> Result<Json<Vec<PertemuanKuliah>>, AppError> {
    dosen_context(&pool, &claims, jadwal_id).await?;
    Ok(Json(repo::get_pertemuan_by_jadwal(&pool, jadwal_id).await?))
}

pub async fn create_pertemuan_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(jadwal_id): Path<Uuid>,
    Json(payload): Json<CreatePertemuanPayload>,
) -> Result<(StatusCode, Json<PertemuanKuliah>), AppError> {
    dosen_context(&pool, &claims, jadwal_id).await?;
    if !(1..=32).contains(&payload.pertemuan_ke) {
        return Err(AppError::BadRequest(
            "Pertemuan harus berada pada rentang 1–32.".to_string(),
        ));
    }
    let row = repo::create_pertemuan(&pool, jadwal_id, payload).await?;
    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn get_detail_pertemuan_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(pertemuan_id): Path<Uuid>,
) -> Result<Json<DetailPertemuanResponse>, AppError> {
    dosen_context_by_pertemuan(&pool, &claims, pertemuan_id).await?;
    Ok(Json(repo::get_detail_pertemuan(&pool, pertemuan_id).await?))
}

pub async fn update_bap_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(pertemuan_id): Path<Uuid>,
    Json(payload): Json<UpdateBapPayload>,
) -> Result<Json<PertemuanKuliah>, AppError> {
    dosen_context_by_pertemuan(&pool, &claims, pertemuan_id).await?;
    Ok(Json(repo::update_bap(&pool, pertemuan_id, payload).await?))
}

pub async fn buka_pertemuan_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(pertemuan_id): Path<Uuid>,
) -> Result<Json<SesiPresensiResponse>, AppError> {
    let dosen_id = dosen_context_by_pertemuan(&pool, &claims, pertemuan_id).await?;
    let kode = Uuid::new_v4().simple().to_string()[..8].to_uppercase();
    Ok(Json(
        repo::buka_pertemuan(&pool, pertemuan_id, dosen_id, claims.sub, kode).await?,
    ))
}

pub async fn tutup_pertemuan_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(pertemuan_id): Path<Uuid>,
) -> Result<Json<SuccessMessage>, AppError> {
    let dosen_id = dosen_context_by_pertemuan(&pool, &claims, pertemuan_id).await?;
    repo::tutup_pertemuan(&pool, pertemuan_id, dosen_id, claims.sub).await?;
    Ok(Json(SuccessMessage {
        message: "Pertemuan berhasil ditutup.".to_string(),
    }))
}

pub async fn manual_presensi_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path((pertemuan_id, enrollment_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<ManualPresensiPayload>,
) -> Result<Json<SuccessMessage>, AppError> {
    dosen_context_by_pertemuan(&pool, &claims, pertemuan_id).await?;
    let allowed = ["Hadir", "Terlambat", "Izin", "Sakit", "Alpa"];
    if !allowed.contains(&payload.status.as_str()) {
        return Err(AppError::BadRequest(
            "Status presensi tidak valid.".to_string(),
        ));
    }
    repo::upsert_manual_presensi(
        &pool,
        pertemuan_id,
        enrollment_id,
        payload.status,
        payload.catatan,
        claims.sub,
    )
    .await?;
    Ok(Json(SuccessMessage {
        message: "Presensi mahasiswa diperbarui.".to_string(),
    }))
}

pub async fn check_in_mahasiswa_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<CheckInMahasiswaPayload>,
) -> Result<Json<SuccessMessage>, AppError> {
    repo::check_in_mahasiswa(&pool, claims.sub, payload.kode).await?;
    Ok(Json(SuccessMessage {
        message: "Presensi berhasil direkam.".to_string(),
    }))
}
