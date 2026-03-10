// src/modules/mahasiswa/handler.rs

use super::{
    model::{
        CreateMahasiswaPayload, ImportResult, MahasiswaDetail, MahasiswaRombelDetail,
        MahasiswaRombelFilter, PindahRombelPayload, RenameRombelPayload, RombelFilter,
        RombelSummary, UpdateMahasiswaPayload,
    },
    repo as mahasiswa_repo,
};
use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};
use axum::response::{IntoResponse, Response};
use axum::{
    Extension,
    extract::{Json, Multipart, Path, Query, State},
    http::StatusCode,
};
use serde_json::json;
use uuid::Uuid;

/// Handler untuk membuat data Mahasiswa baru, sekaligus membuat akun user-nya.
pub async fn create_mahasiswa_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateMahasiswaPayload>,
) -> Result<(StatusCode, Json<MahasiswaDetail>), AppError> {
    // Memanggil fungsi repository yang sudah kita buat (yang berisi transaksi)
    let created_mahasiswa = mahasiswa_repo::create_mahasiswa_repo(&pool, payload).await?;

    Ok((StatusCode::CREATED, Json(created_mahasiswa)))
}

/// Handler untuk mendapatkan semua data Mahasiswa.
pub async fn get_all_mahasiswa_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<MahasiswaDetail>>, AppError> {
    let mahasiswa_list = mahasiswa_repo::get_all_mahasiswa_repo(&pool).await?;
    Ok(Json(mahasiswa_list))
}

// Anda bisa menambahkan handler lain di sini nanti, seperti get_by_id, update, dan delete
// dengan pola yang sama seperti pada dosen_handler.

pub async fn import_mahasiswa_from_csv_handler(
    State(pool): State<DbPool>,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>, AppError> {
    // Cari field file dari request multipart
    if let Some(field) = multipart.next_field().await? {
        // Pastikan fieldnya adalah 'file'
        if field.name() == Some("file") {
            let file_data = field.bytes().await?;
            let result = mahasiswa_repo::import_mahasiswa_from_csv_repo(&pool, file_data).await?;
            return Ok(Json(result));
        }
    }

    // Jika tidak ada field 'file'
    Err(
        anyhow::anyhow!("Request harus menyertakan field 'file' dalam format multipart/form-data")
            .into(),
    )
}

/// Handler untuk mendapatkan detail satu mahasiswa berdasarkan ID
pub async fn get_mahasiswa_by_id_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>, // Axum akan mengekstrak ID dari URL
) -> Result<Json<MahasiswaDetail>, AppError> {
    let mahasiswa = mahasiswa_repo::get_mahasiswa_by_id_repo(&pool, id).await?;
    Ok(Json(mahasiswa))
}

pub async fn update_mahasiswa_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<TokenClaims>, // <-- Ambil info user dari token
    Json(payload): Json<UpdateMahasiswaPayload>,
) -> Result<Json<MahasiswaDetail>, AppError> {
    // Cek apakah ada upaya untuk mengubah NIM
    if let Some(ref _nim) = payload.nim {
        // Jika ada, periksa apakah user memiliki peran yang diizinkan (misal: SUPER_ADMIN)
        if !claims.roles.contains(&"SUPER_ADMIN".to_string()) {
            // Jika tidak, tolak dengan error 403 Forbidden
            return Err(AppError::Forbidden(
                "Hanya SUPER_ADMIN yang dapat mengubah NIM.".to_string(),
            ));
        }
    }

    let updated_mahasiswa = mahasiswa_repo::update_mahasiswa_repo(&pool, id, payload).await?;
    Ok(Json(updated_mahasiswa))
}

pub async fn delete_mahasiswa_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    mahasiswa_repo::delete_mahasiswa_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn download_mahasiswa_csv_template_handler() -> Response {
    // 1. Definisikan header CSV
    let header = "nik;nim;nama_mahasiswa;email;angkatan;kode_prodi";

    // 2. Buat beberapa baris contoh untuk memperjelas format
    let example_row1 =
        "330250101001105;250101001;Budi Darmawan;budi.d@student.kampus.ac.id;2025;S1TI";
    let example_row2 = "330250202002101;250202002;Citra Ayu;citra.a@student.kampus.ac.id;2025;D3MI";

    // 3. Gabungkan menjadi satu string konten CSV
    let content = format!("{}\n{}\n{}", header, example_row1, example_row2);

    // 4. Siapkan header HTTP untuk respons
    let headers = [
        // Memberitahu browser bahwa ini adalah file CSV
        (axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8"),
        // Memberitahu browser untuk mengunduh file dengan nama tertentu
        (
            axum::http::header::CONTENT_DISPOSITION,
            "attachment; filename=\"template_mahasiswa.csv\"",
        ),
    ];

    // 5. Kembalikan respons dengan header dan konten
    (headers, content).into_response()
}

pub async fn get_rombel_summary_handler(
    State(pool): State<DbPool>,
    Query(filter): Query<RombelFilter>,
) -> Result<Json<Vec<RombelSummary>>, AppError> {
    let result = mahasiswa_repo::get_rombel_summary_repo(&pool, filter).await?;
    Ok(Json(result))
}

pub async fn get_mahasiswa_by_rombel_handler(
    State(pool): State<DbPool>,
    Query(filter): Query<MahasiswaRombelFilter>,
) -> Result<Json<Vec<MahasiswaRombelDetail>>, AppError> {
    let result = mahasiswa_repo::get_mahasiswa_by_rombel_repo(&pool, filter).await?;
    Ok(Json(result))
}

pub async fn pindah_rombel_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<PindahRombelPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let count = mahasiswa_repo::pindah_rombel_repo(&pool, payload).await?;
    Ok(Json(json!({
        "message": format!("Berhasil memindahkan {} mahasiswa ke rombel baru.", count)
    })))
}

pub async fn rename_rombel_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<RenameRombelPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let count = mahasiswa_repo::rename_rombel_repo(&pool, payload).await?;
    Ok(Json(json!({
        "message": format!("Berhasil mengubah nama rombel untuk {} mahasiswa.", count)
    })))
}
