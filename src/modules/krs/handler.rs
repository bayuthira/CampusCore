// src/handlers/krs_handler.rs

use crate::{
    auth::TokenClaims,
    db::DbPool,
    errors::AppError,
    models::krs_model::{CreateEnrollmentPayload, EnrollmentDetail, UpdateEnrollmentStatusPayload, KrsQuery},
    repositories::krs_repo,
};
use axum::{
    Extension,
    extract::{Path,Json, Query, State},
    http::StatusCode,
};
use uuid::Uuid;
use crate::models::general_model::SuccessResponse;

// Handler saat mahasiswa mengambil MK

pub async fn create_enrollment_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<CreateEnrollmentPayload>,
) -> Result<(StatusCode, Json<SuccessResponse>), AppError> { // <-- Ubah return type
    let user_id = claims.sub;

    let mhs = sqlx::query!("SELECT id FROM mahasiswa WHERE user_id = $1", user_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| {
            AppError::Forbidden("Akun Anda tidak terdaftar sebagai profil mahasiswa aktif.".to_string())
        })?;

    // Panggil repo. Jika ini berhasil, artinya INSERT sukses.
    krs_repo::create_enrollment_repo(&pool, mhs.id, payload).await?;

    // Buat respons sukses sederhana sesuai permintaan Anda
    let response = SuccessResponse {
        message: "Mata Kuliah berhasil masuk krs".to_string(),
    };
    
    Ok((StatusCode::CREATED, Json(response)))
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
        .ok_or_else(|| {
            AppError::Forbidden(
                "Akun Anda tidak terdaftar sebagai profil mahasiswa aktif.".to_string(),
            )
        })?;

    let enrollments =
        krs_repo::get_my_enrollments_repo(&pool, mhs.id, query.tahun_akademik_id).await?;
    Ok(Json(enrollments))
}

pub async fn delete_enrollment_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>, // Info user yang login
    Path(enrollment_id): Path<Uuid>,           // ID enrollment yang akan dihapus
) -> Result<StatusCode, AppError> {
    // 1. Ambil detail enrollment yang akan dihapus untuk mengetahui siapa pemiliknya
    let enrollment_to_delete = krs_repo::get_enrollment_by_id_repo(&pool, enrollment_id).await?;

    // 2. Dapatkan profil mahasiswa dari user yang sedang login
    let user_id = claims.sub;
    let mhs = sqlx::query!("SELECT id FROM mahasiswa WHERE user_id = $1", user_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::Forbidden("Hanya profil mahasiswa yang bisa menghapus KRS.".to_string()))?;

    // 3. Logika Otorisasi: Cek apakah user adalah pemilik data ATAU seorang SUPER_ADMIN
    if mhs.id != enrollment_to_delete.mahasiswa_id && !claims.roles.contains(&"SUPER_ADMIN".to_string()) {
        // Jika bukan, tolak akses!
        return Err(AppError::Forbidden("Anda tidak berhak menghapus data KRS ini.".to_string()));
    }

    // 4. Jika pengecekan lolos, lanjutkan proses penghapusan
    krs_repo::delete_enrollment_repo(&pool, enrollment_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_enrollment_status_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(enrollment_id): Path<Uuid>,
    Json(payload): Json<UpdateEnrollmentStatusPayload>,
) -> Result<Json<SuccessResponse>, AppError> {
    // 1. Dapatkan ID dosen yang sedang login
    let dosen = sqlx::query!("SELECT id FROM dosen WHERE user_id = $1", claims.sub)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::Forbidden("Hanya profil dosen yang dapat melakukan aksi ini.".to_string()))?;

    // 2. Dapatkan detail enrollment untuk tahu siapa mahasiswa pemiliknya
    let enrollment = krs_repo::get_enrollment_by_id_repo(&pool, enrollment_id).await?;

    // 3. Dapatkan detail mahasiswa tersebut untuk tahu siapa Dosen PA-nya
    let mahasiswa = sqlx::query!("SELECT dosen_pa_id FROM mahasiswa WHERE id = $1", enrollment.mahasiswa_id)
        .fetch_one(&pool)
        .await?;

    // 4. Logika Otorisasi Utama
    if mahasiswa.dosen_pa_id != Some(dosen.id) && !claims.roles.contains(&"SUPER_ADMIN".to_string()) {
        return Err(AppError::Forbidden("Anda bukan Dosen PA untuk mahasiswa ini.".to_string()));
    }

    // 5. Jika lolos otorisasi, lanjutkan update
    krs_repo::update_enrollment_status_repo(&pool, enrollment_id, payload).await?;
    
    let response = SuccessResponse {
        message: "Status KRS berhasil diperbarui.".to_string(),
    };

    Ok(Json(response))
}
