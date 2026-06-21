// src/modules/krs/dosen_pa_handler.rs
use super::{
    model::{EnrollmentDetail, KrsQuery},
    repo as krs_repo,
};

use crate::{
    db::DbPool,
    errors::AppError,
    modules::auth::middleware::TokenClaims,
    modules::general::model::SuccessResponse,
    modules::krs::dosen_pa_repo,
    modules::mahasiswa::model::{
        BatchAssignDosenPaPayload, MahasiswaBimbingan, SingleAssignDosenPaPayload,
    },
};
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;

pub async fn get_my_advisees_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(query): Query<KrsQuery>,
) -> Result<Json<Vec<MahasiswaBimbingan>>, AppError> {
    let logged_in_user_id = claims.sub;

    // --- PERBAIKAN: JOIN ke pegawai untuk mencari dosen berdasarkan user_id ---
    let dosen = sqlx::query!(
        "SELECT d.id FROM dosen d JOIN pegawai p ON d.pegawai_id = p.id WHERE p.user_id = $1",
        logged_in_user_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Forbidden("Hanya dosen yang dapat mengakses data ini.".to_string()))?;

    let advisees =
        dosen_pa_repo::get_my_advisees_repo(&pool, dosen.id, query.tahun_akademik_id).await?;

    Ok(Json(advisees))
}

pub async fn get_advisee_krs_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(mahasiswa_id): Path<Uuid>,
    Query(query): Query<KrsQuery>,
) -> Result<Json<Vec<EnrollmentDetail>>, AppError> {
    // --- PERBAIKAN: JOIN ke pegawai untuk mencari dosen berdasarkan user_id ---
    let dosen = sqlx::query!(
        "SELECT d.id FROM dosen d JOIN pegawai p ON d.pegawai_id = p.id WHERE p.user_id = $1",
        claims.sub
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Forbidden("Hanya dosen yang dapat mengakses data ini.".to_string()))?;

    let registrasi = sqlx::query!(
        "SELECT id, dosen_pa_id FROM registrasi_mahasiswa WHERE mahasiswa_id = $1 ORDER BY created_at DESC LIMIT 1",
        mahasiswa_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Forbidden("Mahasiswa tidak memiliki data registrasi aktif.".to_string()))?;

    // Logika Otorisasi
    if registrasi.dosen_pa_id != Some(dosen.id)
        && !claims.roles.contains(&"SUPER_ADMIN".to_string())
    {
        return Err(AppError::Forbidden(
            "Anda bukan Dosen PA untuk mahasiswa ini.".to_string(),
        ));
    }

    // Panggil repo KRS menggunakan 'registrasi.id'
    let enrollments =
        krs_repo::get_my_enrollments_repo(&pool, registrasi.id, query.tahun_akademik_id).await?;

    Ok(Json(enrollments))
}

// --- HANDLER BARU: BATCH ASSIGN ---
pub async fn batch_assign_dosen_pa_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<BatchAssignDosenPaPayload>,
) -> Result<Json<SuccessResponse>, AppError> {
    let rows_updated = dosen_pa_repo::batch_assign_dosen_pa_repo(&pool, payload).await?;

    Ok(Json(SuccessResponse {
        message: format!(
            "Berhasil menetapkan Dosen PA untuk {} mahasiswa.",
            rows_updated
        ),
    }))
}

// --- HANDLER BARU: SINGLE ASSIGN ---
pub async fn single_assign_dosen_pa_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<SingleAssignDosenPaPayload>,
) -> Result<Json<SuccessResponse>, AppError> {
    dosen_pa_repo::single_assign_dosen_pa_repo(&pool, payload).await?;

    Ok(Json(SuccessResponse {
        message: "Berhasil menetapkan Dosen PA untuk mahasiswa tersebut.".to_string(),
    }))
}
