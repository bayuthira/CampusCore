// src/modules/krs/handler.rs
use super::{
    model::{
        CreateEnrollmentPayload, EnrollmentDetail, KrsQuery, UpdateEnrollmentStatusPayload,
        UpdateNilaiPayload,
    },
    repo as krs_repo,
};

use crate::modules::general::model::SuccessResponse;
use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};
use axum::{
    Extension,
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

pub async fn create_enrollment_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<CreateEnrollmentPayload>,
) -> Result<(StatusCode, Json<SuccessResponse>), AppError> {
    let user_id = claims.sub;

    let mhs = sqlx::query!(
        r#"
        SELECT rm.id as registrasi_id 
        FROM mahasiswa m 
        JOIN registrasi_mahasiswa rm ON m.id = rm.mahasiswa_id 
        WHERE m.user_id = $1 
        ORDER BY rm.created_at DESC LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| {
        AppError::Forbidden("Akun Anda tidak terdaftar sebagai profil mahasiswa aktif.".to_string())
    })?;

    krs_repo::create_enrollment_repo(&pool, mhs.registrasi_id, payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(SuccessResponse {
            message: "Mata Kuliah berhasil masuk krs".to_string(),
        }),
    ))
}

pub async fn get_my_enrollments_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(query): Query<KrsQuery>,
) -> Result<Json<Vec<EnrollmentDetail>>, AppError> {
    let user_id = claims.sub;

    let mhs = sqlx::query!(
        r#"
        SELECT rm.id as registrasi_id 
        FROM mahasiswa m 
        JOIN registrasi_mahasiswa rm ON m.id = rm.mahasiswa_id 
        WHERE m.user_id = $1 
        ORDER BY rm.created_at DESC LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| {
        AppError::Forbidden("Akun Anda tidak terdaftar sebagai profil mahasiswa aktif.".to_string())
    })?;

    let enrollments =
        krs_repo::get_my_enrollments_repo(&pool, mhs.registrasi_id, query.tahun_akademik_id)
            .await?;
    Ok(Json(enrollments))
}

pub async fn delete_enrollment_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(enrollment_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let enrollment_to_delete = krs_repo::get_enrollment_by_id_repo(&pool, enrollment_id).await?;

    let user_id = claims.sub;

    let mhs = sqlx::query!(
        r#"
        SELECT rm.id as registrasi_id 
        FROM mahasiswa m 
        JOIN registrasi_mahasiswa rm ON m.id = rm.mahasiswa_id 
        WHERE m.user_id = $1 
        ORDER BY rm.created_at DESC LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| {
        AppError::Forbidden("Hanya profil mahasiswa yang bisa menghapus KRS.".to_string())
    })?;

    if mhs.registrasi_id != enrollment_to_delete.registrasi_id
        && !claims.roles.contains(&"SUPER_ADMIN".to_string())
    {
        return Err(AppError::Forbidden(
            "Anda tidak berhak menghapus data KRS ini.".to_string(),
        ));
    }

    krs_repo::delete_enrollment_repo(&pool, enrollment_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_enrollment_status_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(enrollment_id): Path<Uuid>,
    Json(payload): Json<UpdateEnrollmentStatusPayload>,
) -> Result<Json<SuccessResponse>, AppError> {
    let dosen = sqlx::query!("SELECT id FROM dosen WHERE user_id = $1", claims.sub)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| {
            AppError::Forbidden("Hanya profil dosen yang dapat melakukan aksi ini.".to_string())
        })?;

    let enrollment = krs_repo::get_enrollment_by_id_repo(&pool, enrollment_id).await?;

    let registrasi = sqlx::query!(
        "SELECT dosen_pa_id FROM registrasi_mahasiswa WHERE id = $1",
        enrollment.registrasi_id
    )
    .fetch_one(&pool)
    .await?;

    if registrasi.dosen_pa_id != Some(dosen.id)
        && !claims.roles.contains(&"SUPER_ADMIN".to_string())
    {
        return Err(AppError::Forbidden(
            "Anda bukan Dosen PA untuk mahasiswa ini.".to_string(),
        ));
    }

    krs_repo::update_enrollment_status_repo(&pool, enrollment_id, payload).await?;

    Ok(Json(SuccessResponse {
        message: "Status KRS berhasil diperbarui.".to_string(),
    }))
}

pub async fn update_nilai_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(enrollment_id): Path<Uuid>,
    Json(payload): Json<UpdateNilaiPayload>,
) -> Result<Json<SuccessResponse>, AppError> {
    let is_admin = claims.roles.contains(&"SUPER_ADMIN".to_string());
    let dosen = sqlx::query!("SELECT id FROM dosen WHERE user_id = $1", claims.sub)
        .fetch_optional(&pool)
        .await?;

    if !is_admin && dosen.is_none() {
        return Err(AppError::Forbidden(
            "Hanya Dosen atau Admin yang bisa menginput nilai.".to_string(),
        ));
    }

    let enrollment = krs_repo::get_enrollment_by_id_repo(&pool, enrollment_id).await?;

    let registrasi = sqlx::query!(
        "SELECT dosen_pa_id FROM registrasi_mahasiswa WHERE id = $1",
        enrollment.registrasi_id
    )
    .fetch_one(&pool)
    .await?;

    if !is_admin {
        if registrasi.dosen_pa_id != Some(dosen.unwrap().id) {
            return Err(AppError::Forbidden(
                "Anda bukan Dosen PA untuk mahasiswa ini.".to_string(),
            ));
        }
    }

    krs_repo::update_nilai_repo(&pool, enrollment_id, payload).await?;

    Ok(Json(SuccessResponse {
        message: "Nilai berhasil diperbarui.".to_string(),
    }))
}
