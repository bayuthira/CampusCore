// src/handlers/krs_handler.rs

use crate::{
    auth::TokenClaims,
    db::DbPool,
    errors::AppError,
    models::krs_model::{CreateEnrollmentPayload, EnrollmentDetail},
    repositories::krs_repo,
};
use axum::{
    extract::{Query, State, Json},
    http::StatusCode,
    Extension,
};
use serde::Deserialize;
use uuid::Uuid;

// Handler saat mahasiswa mengambil MK
pub async fn create_enrollment_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>, // Ambil data user yg login
    Json(payload): Json<CreateEnrollmentPayload>,
) -> Result<(StatusCode, Json<EnrollmentDetail>), AppError> {
    let user_id = claims.sub;

    // Cari mahasiswa_id yang berelasi dengan user_id ini
    let mhs = sqlx::query!("SELECT id FROM mahasiswa WHERE user_id = $1", user_id)
        .fetch_optional(&pool)
        .await?
        // Jika user yang login tidak punya profil mahasiswa, tolak.
        .ok_or(AppError::Forbidden)?;

    let enrollment = krs_repo::create_enrollment_repo(&pool, mhs.id, payload).await?;
    Ok((StatusCode::CREATED, Json(enrollment)))
}

// Struct untuk query parameter di URL, contoh: ?tahun_akademik_id=...
#[derive(Debug, Deserialize)]
pub struct KrsQuery {
    pub tahun_akademik_id: Uuid,
}

// Handler untuk mahasiswa melihat KRS miliknya di periode tertentu
pub async fn get_my_enrollments_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(query): Query<KrsQuery>, // Ambil tahun_akademik_id dari query param
) -> Result<Json<Vec<EnrollmentDetail>>, AppError> {
    let user_id = claims.sub;

    let mhs = sqlx::query!("SELECT id FROM mahasiswa WHERE user_id = $1", user_id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::Forbidden)?;

    let enrollments =
        krs_repo::get_my_enrollments_repo(&pool, mhs.id, query.tahun_akademik_id).await?;
    Ok(Json(enrollments))
}