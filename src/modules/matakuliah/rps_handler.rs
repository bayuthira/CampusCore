// src/modules/matakuliah/rps_handler.rs
use super::{
    rps_access,
    rps_model::{
        RpsHeaderDetail, RpsMingguanDetail, UpsertRpsHeaderPayload, UpsertRpsMingguanPayload,
    },
    rps_repo,
};
use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Extension,
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

pub async fn get_rps_mata_kuliah_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
) -> Result<Json<Vec<super::rps_model::RpsMataKuliahAccess>>, AppError> {
    Ok(Json(rps_access::list_for_user(&pool, &claims).await?))
}

pub async fn get_rps_header_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(mata_kuliah_id): Path<Uuid>,
) -> Result<Json<Option<RpsHeaderDetail>>, AppError> {
    rps_access::assert_can_view(&pool, &claims, mata_kuliah_id).await?;
    let header = rps_repo::get_rps_header_repo(&pool, mata_kuliah_id).await?;
    Ok(Json(header))
}

pub async fn upsert_rps_header_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(mata_kuliah_id): Path<Uuid>,
    Json(payload): Json<UpsertRpsHeaderPayload>,
) -> Result<Json<RpsHeaderDetail>, AppError> {
    rps_access::assert_can_edit(&pool, &claims, mata_kuliah_id).await?;
    let header = rps_repo::upsert_rps_header_repo(&pool, mata_kuliah_id, payload).await?;
    Ok(Json(header))
}

// --- HANDLER MINGGUAN RPS ---

pub async fn get_rps_mingguan_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(mata_kuliah_id): Path<Uuid>,
) -> Result<Json<Vec<RpsMingguanDetail>>, AppError> {
    rps_access::assert_can_view(&pool, &claims, mata_kuliah_id).await?;
    let list = rps_repo::get_rps_mingguan_repo(&pool, mata_kuliah_id).await?;
    Ok(Json(list))
}

pub async fn upsert_rps_mingguan_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(mata_kuliah_id): Path<Uuid>,
    Json(payload): Json<UpsertRpsMingguanPayload>,
) -> Result<Json<RpsMingguanDetail>, AppError> {
    rps_access::assert_can_edit(&pool, &claims, mata_kuliah_id).await?;
    let mingguan = rps_repo::upsert_rps_mingguan_repo(&pool, mata_kuliah_id, payload).await?;
    Ok(Json(mingguan))
}

pub async fn delete_rps_mingguan_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id_mingguan): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let mata_kuliah_id = rps_access::mata_kuliah_id_for_weekly(&pool, id_mingguan).await?;
    rps_access::assert_can_edit(&pool, &claims, mata_kuliah_id).await?;
    rps_repo::delete_rps_mingguan_repo(&pool, id_mingguan).await?;
    Ok(StatusCode::NO_CONTENT)
}

// --- HANDLER CETAK/PRINT OUT RPS TERSTRUKTUR ---

pub async fn print_rps_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(mata_kuliah_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    rps_access::assert_can_view(&pool, &claims, mata_kuliah_id).await?;
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
