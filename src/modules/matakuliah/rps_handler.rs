// src/modules/matakuliah/rps_handler.rs
use super::{
    rps_model::{
        RpsHeaderDetail, RpsMingguanDetail, UpsertRpsHeaderPayload, UpsertRpsMingguanPayload,
    },
    rps_repo,
};
use crate::{db::DbPool, errors::AppError};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use lazy_static::lazy_static;
use tera::{Context, Tera};
use uuid::Uuid;

// Inisialisasi Tera secara global untuk RPS
lazy_static! {
    pub static ref TERA_RPS: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["html"]);
        tera
    };
}

// Struct penampung (Context) untuk Template Engine Tera
#[derive(serde::Serialize)]
struct RpsTemplateContext {
    mk: super::model::MataKuliahDetail,
    header: Option<RpsHeaderDetail>,
    mingguan: Vec<RpsMingguanDetail>,
}

// --- HANDLER HEADER RPS ---

pub async fn get_rps_header_handler(
    State(pool): State<DbPool>,
    Path(mata_kuliah_id): Path<Uuid>,
) -> Result<Json<Option<RpsHeaderDetail>>, AppError> {
    let header = rps_repo::get_rps_header_repo(&pool, mata_kuliah_id).await?;
    Ok(Json(header))
}

pub async fn upsert_rps_header_handler(
    State(pool): State<DbPool>,
    Path(mata_kuliah_id): Path<Uuid>,
    Json(payload): Json<UpsertRpsHeaderPayload>,
) -> Result<Json<RpsHeaderDetail>, AppError> {
    let header = rps_repo::upsert_rps_header_repo(&pool, mata_kuliah_id, payload).await?;
    Ok(Json(header))
}

// --- HANDLER MINGGUAN RPS ---

pub async fn get_rps_mingguan_handler(
    State(pool): State<DbPool>,
    Path(mata_kuliah_id): Path<Uuid>,
) -> Result<Json<Vec<RpsMingguanDetail>>, AppError> {
    let list = rps_repo::get_rps_mingguan_repo(&pool, mata_kuliah_id).await?;
    Ok(Json(list))
}

pub async fn upsert_rps_mingguan_handler(
    State(pool): State<DbPool>,
    Path(mata_kuliah_id): Path<Uuid>,
    Json(payload): Json<UpsertRpsMingguanPayload>,
) -> Result<Json<RpsMingguanDetail>, AppError> {
    let mingguan = rps_repo::upsert_rps_mingguan_repo(&pool, mata_kuliah_id, payload).await?;
    Ok(Json(mingguan))
}

pub async fn delete_rps_mingguan_handler(
    State(pool): State<DbPool>,
    Path(id_mingguan): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    rps_repo::delete_rps_mingguan_repo(&pool, id_mingguan).await?;
    Ok(StatusCode::NO_CONTENT)
}

// --- HANDLER CETAK/PRINT OUT RPS TERSTRUKTUR ---

pub async fn print_rps_handler(
    State(pool): State<DbPool>,
    Path(mata_kuliah_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Tarik 3 Data Utama: MK, Header RPS, dan Mingguan
    let mk = super::repo::get_matakuliah_by_id_repo(&pool, mata_kuliah_id).await?;
    let header = rps_repo::get_rps_header_repo(&pool, mata_kuliah_id).await?;
    let mingguan = rps_repo::get_rps_mingguan_repo(&pool, mata_kuliah_id).await?;

    // Bungkus menjadi satu Context
    let data = RpsTemplateContext {
        mk,
        header,
        mingguan,
    };
    let context = Context::from_serialize(&data).map_err(|_| {
        AppError::InternalServerError("Gagal mem-parsing data RPS untuk cetak".into())
    })?;

    // Render file `templates/rps.html`
    let rendered_html = TERA_RPS
        .render("rps.html", &context)
        .map_err(|e| AppError::InternalServerError(format!("Template error: {}", e)))?;

    Ok(Html(rendered_html))
}
