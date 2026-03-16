// src/modules/akademik/jadwal_kuliah_handler.rs
use super::{
    jadwal_kuliah_model::{
        CreateJadwalKuliahPayload, ImportJadwalResult, JadwalKuliahDetail, JadwalKuliahFilter,
        PlotJadwalRuanganPayload, UpdateJadwalKuliahPayload,
    },
    jadwal_kuliah_repo,
};
use crate::modules::auth::middleware::TokenClaims;
use crate::{db::DbPool, errors::AppError, modules::general::model::SuccessResponse};
use axum::Extension;
use axum::{
    extract::{Json, Multipart, Path, Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use uuid::Uuid;

pub async fn create_jadwal_kuliah_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateJadwalKuliahPayload>,
) -> Result<(StatusCode, Json<SuccessResponse>), AppError> {
    let jadwal_id = jadwal_kuliah_repo::create_jadwal_kuliah_repo(&pool, payload).await?;
    Ok((
        StatusCode::CREATED,
        Json(SuccessResponse {
            message: format!("Jadwal kuliah berhasil dibuat dengan ID: {}", jadwal_id),
        }),
    ))
}

pub async fn plot_jadwal_ruangan_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<PlotJadwalRuanganPayload>,
) -> Result<Json<SuccessResponse>, AppError> {
    let user_pembuat_id = claims.sub;
    jadwal_kuliah_repo::plot_jadwal_ruangan_repo(&pool, user_pembuat_id, payload).await?;
    Ok(Json(SuccessResponse {
        message: "Jadwal kuliah berhasil di-plot ke ruangan untuk satu semester.".to_string(),
    }))
}

pub async fn unplot_jadwal_ruangan_handler(
    State(pool): State<DbPool>,
    Path(jadwal_kuliah_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    jadwal_kuliah_repo::unplot_jadwal_ruangan_repo(&pool, jadwal_kuliah_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_all_jadwal_kuliah_handler(
    State(pool): State<DbPool>,
    Query(filter): Query<JadwalKuliahFilter>,
) -> Result<Json<Vec<JadwalKuliahDetail>>, AppError> {
    let jadwal_list = jadwal_kuliah_repo::get_all_jadwal_kuliah_repo(&pool, filter).await?;
    Ok(Json(jadwal_list))
}

pub async fn update_jadwal_kuliah_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateJadwalKuliahPayload>,
) -> Result<Json<SuccessResponse>, AppError> {
    jadwal_kuliah_repo::update_jadwal_kuliah_repo(&pool, id, payload).await?;
    Ok(Json(SuccessResponse {
        message: format!("Jadwal kuliah dengan ID {} berhasil diperbarui.", id),
    }))
}

pub async fn delete_jadwal_kuliah_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    jadwal_kuliah_repo::delete_jadwal_kuliah_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// =======================================================
// --- HANDLER BARU UNTUK IMPORT CSV ---
// =======================================================

pub async fn download_jadwal_csv_template_handler() -> impl IntoResponse {
    let header_csv =
        "\"Hari\";\"Jam\";\"Kode MK\";\"Dosen Pengampu\";\"Kelas\";\"Ruangan\";\"tahun akademik\"";
    let row_contoh = "\"Senin\";\"08:00 - 11:40\";\"MKP3204\";\"Rhela Panji Raraswati, S.Tr.Keb., Bdn., M.Tr.Keb - Bdn. Annisa Rahmidini, S.ST., M.Keb - Chanty Yunie HR, SST., M.Kes\";\"s1 kebidanan 2025\";\"Ruang Kelas R.III.I\";20252";
    let csv_content = format!("{}\n{}\n", header_csv, row_contoh);

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"template_import_jadwal.csv\"",
            ),
        ],
        csv_content,
    )
}

pub async fn import_jadwal_csv_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ImportJadwalResult>), AppError> {
    let user_pembuat_id = claims.sub;

    if let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            let file_data = field.bytes().await?;
            let result =
                jadwal_kuliah_repo::import_jadwal_from_csv_repo(&pool, file_data, user_pembuat_id)
                    .await?;
            return Ok((StatusCode::OK, Json(result)));
        }
    }

    Err(AppError::BadRequest(
        "Request harus menyertakan field 'file' dalam format multipart/form-data".to_string(),
    ))
}
