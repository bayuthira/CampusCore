// src/handlers/dosen_pa_handler.rs
use super::{
    model::{EnrollmentDetail, KrsQuery},
    repo as krs_repo,
};                    
                      
use uuid::Uuid;                                          
use crate::{
    modules::auth::middleware::TokenClaims,
    db::DbPool,
    errors::AppError,
    modules::mahasiswa::model::MahasiswaBimbingan,
    modules::krs::dosen_pa_repo,
};
use axum::{extract::{Path, Query,State}, Json, Extension};

/// Handler untuk Dosen PA melihat daftar mahasiswa bimbingannya
pub async fn get_my_advisees_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>, // Ambil info user yang login
) -> Result<Json<Vec<MahasiswaBimbingan>>, AppError> {
    // 1. Dapatkan user_id dari token
    let logged_in_user_id = claims.sub;

    // 2. Cari profil dosen yang terhubung dengan user ini
    let dosen = sqlx::query!("SELECT id FROM dosen WHERE user_id = $1", logged_in_user_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::Forbidden("Hanya dosen yang dapat mengakses data ini.".to_string()))?;

    // 3. Panggil repository dengan ID dosen yang ditemukan
    let advisees = dosen_pa_repo::get_my_advisees_repo(&pool, dosen.id).await?;

    Ok(Json(advisees))
}

pub async fn get_advisee_krs_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>, // Info Dosen PA yang login
    Path(mahasiswa_id): Path<Uuid>,            // ID Mahasiswa yang ingin dilihat
    Query(query): Query<KrsQuery>,             // ID Tahun Akademik dari query param
) -> Result<Json<Vec<EnrollmentDetail>>, AppError> {
    // 1. Dapatkan ID dosen yang sedang login
    let dosen = sqlx::query!("SELECT id FROM dosen WHERE user_id = $1", claims.sub)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::Forbidden("Hanya dosen yang dapat mengakses data ini.".to_string()))?;

    // 2. Dapatkan detail mahasiswa untuk memeriksa siapa Dosen PA-nya
    let mahasiswa = sqlx::query!(
        "SELECT dosen_pa_id FROM mahasiswa WHERE id = $1",
        mahasiswa_id
    )
    .fetch_one(&pool)
    .await?;

    // 3. Logika Otorisasi: Tolak jika dosen yang login bukan PA dari mahasiswa ini
    //    (KECUALI jika dia adalah SUPER_ADMIN)
    if mahasiswa.dosen_pa_id != Some(dosen.id) && !claims.roles.contains(&"SUPER_ADMIN".to_string()) {
        return Err(AppError::Forbidden(
            "Anda bukan Dosen PA untuk mahasiswa ini.".to_string(),
        ));
    }

    // 4. Jika otorisasi lolos, panggil repo yang sudah ada untuk mengambil data KRS
    let enrollments =
        krs_repo::get_my_enrollments_repo(&pool, mahasiswa_id, query.tahun_akademik_id).await?;

    Ok(Json(enrollments))
}